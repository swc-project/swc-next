export interface UserConfig {
  base?: string | false;
}
export type Base = UserConfig["base"];
export declare function resolveBaseUrl(base: UserConfig["base"] | undefined, isBuild: boolean): string;
export declare function resolveAliasedBaseUrl(base: Base | undefined, isBuild: boolean): string;
