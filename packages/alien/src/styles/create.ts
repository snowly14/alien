import { color } from "../tokens/color";
import type { Style } from "./style";

function create<const Name extends string>(
  _styles: Readonly<Record<Name, Style>>,
): Readonly<Record<Name, string>> {
  throw new Error("alien.create() must be compiled by Alien");
}

export const alien = { create, color };
