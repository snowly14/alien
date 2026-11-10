import { alien, tokens } from "alien";

export const styles = alien.create({
  button: {
    backgroundColor: tokens.color.green[600],
    on: {
      hover: {
        backgroundColor: tokens.color.green[700],
      },
    },
  },
});
