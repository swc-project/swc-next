# TsBinder 实现约定

本文记录 `swc_ecma_semantic2` 单文件 TypeScript 绑定分析的约定。

## 容器与声明身份

- 合法合并的异种声明共用 canonical `SymbolId`，各次声明保留独立的
  `DeclarationId`、flags 和 binding scope。非法冲突的恢复策略另行确定。
- 共享成员表以 owner `SymbolId` 为 key。
- 每个 enum/namespace body 保留自己的 scope、parent 和词法表。
  body 种类复用 `ScopeKind`；`container_owners` 只记录 owner。
- 导出的容器以 canonical export symbol 作为 owner。构建过程中若归并 symbol，
  必须同步 canonicalize owner 和成员索引。

## Enum / namespace 成员贡献

`TsMember` 的两组 flags 按贡献来源划分。

```ts
namespace N {
  export enum E {
    A,
  }
}
```

`N` 表中的 `E` 使用 `namespace_flags`，其中包含 enum 声明的 flags；
`E` 表中的 `A` 才使用 `enum_flags`。

- enum body 的裸名查找使用 enum 贡献，namespace body 使用 namespace 贡献。
  两者的局部词法绑定仍参与查找。
- 通过容器访问公开成员时，按请求的声明空间检查相应的公开贡献。
- 同名成员合法合并时，共用成员 symbol，但分别累计两种来源的 flags。
  查找使用当前 body 可见的贡献 flags。
- namespace 非导出局部声明只进入当前 body 的词法表；隐式导出按导出规则处理。
  ambient namespace 中，普通成员可隐式导出。独立的 export 声明或赋值会关闭
  当前 body 的隐式导出。带 `export` 修饰的声明仍可与隐式导出共存。
- 先判断贡献是否为 `Some`，再调用 `visible_in`。不能把 `None` 转成空 flags：
  当前 `visible_in(ReferenceSpace::Any)` 对空 flags 也返回 true。
- 成员 flags 描述直接贡献。import alias 的目标含义和 type-only 路径由引用解析计算。

## Local / export 双 symbol

直接导出的普通声明具有 local 和 export 两个 symbol：

```text
body 1 locals[name] → local 1 ─┐
                              ├→ canonical export symbol
body 2 locals[name] → local 2 ─┘
container_members[owner][name] → canonical export symbol
```

- export symbol 保存真实声明 flags。导出声明对 local 的 flags 贡献为：有 value
  meaning 时为 `EXPORT_VALUE`，否则为空。
- `EXPORT_VALUE` 是本地 value 入口标记，与声明修饰属性 `EXPORTED` 不同。
  类型/namespace 查找需要查找容器公开成员。
- 一个直接导出声明只创建一个真实 `DeclarationId`，其 `Declaration.symbol`
  指向 export symbol，scope 保留实际 binding scope；声明节点的主要 symbol
  映射也指向 export symbol。
- local 和 export 复用现有 `Symbol.declaration`、`redeclarations` 索引关联这些
  declaration IDs。local 只关联本 body 的声明，
  export 关联合并后的全部声明，均保留源码顺序。
- local 关联的 declaration 由 export symbol 拥有；声明查询和冲突检查需支持
  关联 symbol 与 `Declaration.symbol` 不同。
- local 与 export 的声明冲突分别按各自的 flags 和声明范围处理；每个源码声明
  只统计一次。
- `export { x as y }` 和 `export import A = ...` 按 alias 处理。default 使用下文的绑定规则。

## Import alias

- `import_aliases` 记录 alias 的目标解析结果。缺少 entry 表示该
  symbol 不属于此表管理的 import alias。引用保留 alias 身份。
- `Target(symbol)` 表示直接目标身份已知，目标本身可以是 alias；可见性沿目标链
  计算。
- `Unknown` 表示无法解析目标。引用仍可在该 alias 允许的声明空间命中其词法绑定。
- 构建期间的未解析、正在解析、已完成状态以及有效声明空间只保存在 binder
  临时数据中。`Semantic::lookup` 提供词法查找。transform 对已有 AST 引用读取
  `Reference.symbol`，其中已考虑限定路径上的可见性。
- type-only 语法属性使用 `SymbolFlags::TYPE_IMPORT`，沿链查询时累计各节点限制。
  解析 `N.X` 时也检查限定路径上 `N` 的 flags。经过 type-only alias 后移除
  `VALUE`，保留 `TYPEOF`，已知目标仍记为 `Target(X)`。
- `export import A = ...` 参与 import alias 解析。普通 export specifier 当前只
  解析本地引用，其公开 alias target 暂不建模。
- ES import 和 `import A = require(...)` 提供词法绑定，其可见性由 import flags 决定。
  经由它们访问文件外的限定成员时，成员目标记为 `Unknown`。

## 引用与反向索引

- `Reference.symbol` 保留实际查找命中的身份。
  查询 local 的 value 含义时，对 `EXPORT_VALUE` entry 沿 `export_symbols` 跳转。
- 查找在首个可见绑定处停止。找不到绑定时，`Reference.symbol` 为 `None`。
- `uses(symbol)` 按引用实际命中的 symbol 建立，local/export 分别统计。

## Type parameters

- 每次声明的 type parameters 使用独立 symbol，存放在各自的词法表中。
  interface 合并、class/interface 合并和函数重载均保留这一规则。
- 引用按声明空间解析；type parameter 不遮蔽外层的运行时 value 绑定。
- 当前以 TS → JS transform 为目标，跨声明的泛型身份统一留待类型分析需要时处理。

## Namespace 实例化

每次 namespace 声明的实例化状态保存在 `Declaration.flags`：

| 状态            | Flags                                       |
| --------------- | ------------------------------------------- |
| NonInstantiated | `NAMESPACE_MODULE`，无 `VALUE_MODULE`       |
| ConstEnumOnly   | `VALUE_MODULE` + `CONST_ENUM_ONLY_MODULE`   |
| Instantiated    | `VALUE_MODULE`，无 `CONST_ENUM_ONLY_MODULE` |

- `CONST_ENUM_ONLY_MODULE` 仅保存在 declaration 上。创建或更新 symbol 和共享
  member 的聚合 flags 时过滤该标记；`VALUE_MODULE` 正常参与合并。
- transform 按具体声明读取状态，并结合 ambient 属性和 const enum 配置决定发射。
  同一 owner 下的不同 namespace 声明分别处理。
- 每个 namespace body 按语法计算实例化状态，嵌套 namespace 贡献其 body 的状态。
  value 形式的 `export import` 使 body 实例化。引用是否读取运行时值由 alias 解析决定。

## Source module 导出

- 当前单文件 transform 通过文件词法表和 `export_symbols` 管理顶层直接导出。
- 同名直接导出声明通过 local entry 的 export 链接查找已有 export symbol。
- 名称查找在 local 含义不匹配请求空间时，检查关联 export 的相应声明贡献。
  类型导出的 local flags 可以为空，类型含义保存在 export symbol 上。
- 文件是否为 module 使用现有 source type。公开 export alias 和模块 exports
  查询不属于当前实现范围。

## Class / function 属性

- import-equals 先按 namespace 空间解析左侧。裸名在没有 namespace 绑定时，
  再按 `Any` 查找。
- 限定成员查找使用 namespace 和 enum 的共享成员，以及点号赋值为函数声明引入的
  value 属性。收齐普通声明后、解析别名前写入成员表。已有声明优先于 expando。
- 计算属性和 class 静态成员尚未建模。找不到成员时，目标记为 `Unknown`。

## Declare global

- semantic2 将声明保留在各自的块内。使用 semantic2 的 transformer 擦除
  `declare global` 块，并按块外的词法绑定决定发射。
- 如果 `Mode` 只在全局增补中声明，枚举初始化式中的 `Mode.Next` 保留为
  运行时表达式。如果 `X` 只在全局增补中声明，保留 `export default X`。
- 这些结果也适用于通过 `tsc --isolatedModules` 检查的输入。单文件发射
  采用词法可见的声明，因此无需为上述行为注入全局声明或合并增补块。

## Export default

- 具名直接声明按本地声明绑定，使用声明名和真实声明 flags。每次声明保留
  `EXPORTED | IS_DEFAULT`。
- 匿名直接声明正常分析内部 scope。transform 从 AST 读取默认导出声明。
- 表达式形式的 `export default x` 保留 `x` 的绑定身份。解析到的绑定仅有类型含义时，
  该引用不计为运行时读取。

## Export equals

- `export =` 的目标按普通引用保留绑定身份，使用 `ReferenceSpace::Any` 解析名称。
- 根据目标的 value 含义判断运行时使用；仅有类型含义的目标不计为运行时读取，
  无法确定的 alias 保守处理。
- transform 从 AST 的 `ExportAssignment` 获取导出表达式，结合目标含义和配置
  完成模块转换。
