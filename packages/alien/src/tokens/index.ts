import { breakpoint } from "./breakpoint";
import { color } from "./color";
import { ease } from "./ease";
import { fontSize } from "./font-size";
import { fontWeight } from "./font-weight";
import { radius } from "./radius";
import { space } from "./space";

// These references use the names defined in the Rust crate alien-tokens.
export const tokens = { space, radius, fontWeight, ease, breakpoint, color, fontSize } as const;
