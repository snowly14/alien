import { alien } from "alien";
import { rem } from "alien/values";

export default alien.globals({
  mode: "strict",
  space: { buttonInline: rem(0.625) },
  radius: { button: rem(0.625) },
});
