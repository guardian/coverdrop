/** THIS works around EuiTooltip not showing on hover because of React.StrictMode
 * see https://github.com/elastic/eui/issues/7774
 * We don't want to lose strict mode as it helps catch UI bugs before production */
import { EuiToolTip, EuiToolTipProps } from "@elastic/eui";

export const ToolTip = (props: Partial<EuiToolTipProps>) => {
  return (
    <EuiToolTip
      {...props}
      repositionOnScroll
      anchorProps={{
        ...props.anchorProps,
        onMouseEnter: (e) => {
          if (process.env.NODE_ENV === "development") {
            props.anchorProps?.onMouseEnter?.(e);
            requestAnimationFrame(() =>
              window.dispatchEvent(new Event("scroll")),
            );
          }
        },
      }}
    />
  );
};
