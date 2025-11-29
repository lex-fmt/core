import { cn } from '@/lib/utils';
import { forwardRef, HTMLAttributes } from 'react';

/**
 * ButtonGroup - visually groups buttons together with connected borders.
 *
 * Children should be buttons or ButtonGroupSeparator components.
 * The first button gets rounded left corners, the last gets rounded right corners,
 * and middle buttons have no corner rounding for a seamless connected appearance.
 */
export const ButtonGroup = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
    ({ className, ...props }, ref) => (
        <div
            ref={ref}
            className={cn(
                "inline-flex items-center",
                // Remove border-radius from middle children via CSS
                "[&>*:not(:first-child):not(:last-child)]:rounded-none",
                "[&>*:first-child]:rounded-r-none",
                "[&>*:last-child]:rounded-l-none",
                // Handle separators - they shouldn't affect the rounding logic
                "[&>*:first-child:has(+[data-separator])]:rounded-r-none",
                className
            )}
            {...props}
        />
    )
);
ButtonGroup.displayName = 'ButtonGroup';

/**
 * Visual separator between buttons in a ButtonGroup.
 * Renders as a thin vertical line.
 */
export const ButtonGroupSeparator = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
    ({ className, ...props }, ref) => (
        <div
            ref={ref}
            data-separator
            className={cn("w-px h-5 bg-border", className)}
            {...props}
        />
    )
);
ButtonGroupSeparator.displayName = 'ButtonGroupSeparator';
