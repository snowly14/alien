import { alien, css, tokens } from "alien";
import { alpha, percent, px, rem } from "alien/values";

const styles = alien.create({
  button: {
    backgroundColor: tokens.color.white,
    color: tokens.color.black,
    paddingInline: tokens.space.buttonInline,
    height: rem(2),
    borderRadius: tokens.radius.button,
    fontSize: tokens.fontSize.sm,
    fontWeight: tokens.fontWeight.medium,
    cursor: css.cursor.pointer,
    outlineStyle: css.outlineStyle.solid,
    outlineWidth: px(2),
    outlineColor: alpha(tokens.color.neutral[200], percent(0)),
    transition: {
      property: [
        css.transitionProperty.backgroundColor,
        css.transitionProperty.translate,
        css.transitionProperty.outlineColor,
      ],
      duration: tokens.duration.fast,
      ease: tokens.ease.inOut,
    },
    border: {
      width: px(1),
      style: css.borderStyle.solid,
      color: tokens.color.neutral[200],
    },
    on: {
      hover: {
        backgroundColor: tokens.color.neutral[100],
      },
      active: {
        translate: { y: px(1), x: px(0) },
      },
      focusVisible: {
        outlineColor: alpha(tokens.color.neutral[200], percent(50)),
      },
    },
  },
});

export function Button() {
  return (
    <button type="button" className={styles.button}>
      Outline
    </button>
  );
}
