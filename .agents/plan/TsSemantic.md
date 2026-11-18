# TsBinder 实现约定

本文补充 `crates/swc_ecma_semantic2/src/binder/ts.rs` 未展开的实现细节。
目前只完成数据结构定义，声明绑定、引用解析与最终 `Semantic` 存储尚未接入。

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
- 先判断贡献是否为 `Some`，再调用 `visible_in`。不能把 `None` 转成空 flags：
  当前 `visible_in(ReferenceSpace::Any)` 对空 flags 也返回 true。
- 当前 flags 结构只覆盖直接声明；alias 的确定性和 type-only 路径尚待设计。

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
- `export { x as y }` 和 `export import A = ...` 按 alias 处理；匿名 default 声明没有 local entry。

## Import alias

- `import_aliases` 记录 alias 的直接目标或可能的声明空间。缺少 entry 表示该
  symbol 不属于此表管理的 import alias。引用保留 alias 身份。
- `Target(symbol)` 表示直接目标身份已知，目标本身可以是 alias；可见性沿目标链
  计算，目标身份已知时仍可能存在空间不确定性。
- `Unknown(spaces)` 表示没有已知目标。包含的空间可能可见，查找时加入 candidate
  并继续向外查找；未包含的空间不可见。空集合表示没有可用目标。
- 构建期间的未解析、正在解析、已完成状态单独存储。
- type-only 语法属性使用 `SymbolFlags::TYPE_IMPORT`，沿链查询时累计各节点限制。
  `VALUE` 与 `TYPEOF` 分别记录运行时引用和类型查询的可见性。
- `export import A = ...` 参与 import alias 解析。普通 export specifier 当前只
  解析本地引用，其公开 alias target 暂不建模。

## 引用与反向索引

- `Reference.symbol` 保留实际查找命中的身份。
  查询 local 的 value 含义时，对 `EXPORT_VALUE` entry 沿 `export_symbols` 跳转。
- 原始 `uses(symbol)` 按引用实际命中的 symbol 或 candidate 建立；local/export
  分别统计。需要整个导出实体的 uses 时再提供聚合查询，其 API 尚未设计。
- 不确定查找遇到可能可见的绑定后，继续收集外层 fallback；遇到确定可见的绑定
  才停止。只有一个 candidate 仍可能不确定。
- 同一引用通过多个入口找到同一 symbol 时去重，每个 `(symbol, reference)`
  在 uses 中只出现一次。
- finalize 建立 uses 的计数和填充阶段都包含 definite target 和 candidates。
- candidate 顺序是否保留最近优先尚未确定；不要让调用方提前依赖排序规则。

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
- 状态计算过程使用 binder 临时数据。嵌套 namespace 和 import-equals 的状态
  传播规则尚待细化。

## Source module 导出

- 当前单文件 transform 通过文件词法表和 `export_symbols` 管理顶层直接导出。
- 同名直接导出声明通过 local entry 的 export 链接查找已有 export symbol。
- 名称查找在 local 含义不匹配请求空间时，检查关联 export 的相应声明贡献。
  类型导出的 local flags 可以为空，类型含义保存在 export symbol 上。
- 文件是否为 module 使用现有 source type。公开 export alias 和模块 exports
  查询不属于当前实现范围。

## Class / function 属性

- import-equals 的限定名先按 namespace 空间解析左侧。
- 对具有 namespace 含义的合并 class/function，限定成员查找保留已知的
  namespace 成员贡献；额外的静态属性或函数属性按可能的 value / typeof
  贡献保守处理，并结合 alias 路径限制计算可见性。
- 没有已知目标的可能属性使用 `TsImportAlias::Unknown` 表示。
  已知贡献与可能属性并存时的结果表达，留待 alias 解析接入时确定。
- 属性贡献仅用于限定成员查找，namespace body 的裸名查找使用 namespace
  声明贡献。当前不收集 class 静态成员或 function 属性事实。

## Declare global

- 当前 transform 直接擦除 `declare global`，全局声明注入、跨块合并和专门的
  owner 建模不属于当前实现范围。

## Export default

- 具名直接声明使用 local/export 双 symbol：local 使用声明名，export 使用
  `default`，通过 `export_symbols` 关联。对声明名的引用保留 local 身份。
- 匿名直接声明只有 export symbol。
- 表达式形式的 `export default x` 将 `x` 作为普通引用解析，保留其现有绑定身份。

## Export equals

- `export =` 的目标按普通引用保留绑定身份，使用 `ReferenceSpace::Any` 解析名称。
- 根据目标的 value 含义判断运行时使用；仅有类型含义的目标不计为运行时读取，
  无法确定的 alias 保守处理。
- transform 从 AST 的 `ExportAssignment` 获取导出表达式，结合目标含义和配置
  完成模块转换。
