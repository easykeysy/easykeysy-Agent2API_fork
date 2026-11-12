/**
 * AI use to （ SVG）。
 *
 * ── Why not icons.js ─────────────────────────────────────
 * `ui/icons.js` is vanilla （`window.wbIcons.icon(name, size)` return
 * HTML ）， this can `dangerouslySetInnerHTML` ； need 
 * this 6 （ / / / / / ） in that no has 。 by 
 * `models-page.tsx` （ can that ） in ， 1 ：
 * 24 、currentColor、 1.8~2、。
 *
 * ## 
 * - `PlusIcon`： down 「」 
 * - `ImageIcon`：、 can 
 * - `FileIcon`：JSON / .info file
 * - `ShieldIcon`：「 before 」
 * - `BoltIcon`：「」
 * - `SendIcon`：（ to up ， and ）
 * - `ChevronIcon`： / 
 * - `MiniChevronIcon`： to 
 */

type IconProps = { size?: number; className?: string }

export function PlusIcon({ size = 17, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M12 5.5v13M5.5 12h13' fill='none' stroke='currentColor' strokeWidth='1.9' strokeLinecap='round' />
    </svg>
  )
}

export function ImageIcon({ size = 15, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <g fill='none' stroke='currentColor' strokeWidth='1.7' strokeLinecap='round' strokeLinejoin='round'>
        <rect x='3.5' y='4.5' width='17' height='15' rx='2.6' />
        <circle cx='8.7' cy='9.4' r='1.5' fill='currentColor' stroke='none' />
        <path d='M5.2 17.2l4.7-4.7a1.7 1.7 0 0 1 2.4 0l1.5 1.5 2.1-2.1a1.7 1.7 0 0 1 2.4 0l1.8 1.8' />
      </g>
    </svg>
  )
}

export function FileIcon({ size = 15, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <g fill='none' stroke='currentColor' strokeWidth='1.7' strokeLinecap='round' strokeLinejoin='round'>
        <path d='M6.2 3.5h7.1l4.5 4.5v12.5H6.2z' />
        <path d='M13.3 3.7v4.4h4.4M9 12h6M9 15.3h6' />
      </g>
    </svg>
  )
}

export function ShieldIcon({ size = 15, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M12 3.2l7.2 2.7v5.4c0 4.4-2.9 7.6-7.2 9.2-4.3-1.6-7.2-4.8-7.2-9.2V5.9L12 3.2z'
        fill='none' stroke='currentColor' strokeWidth='1.7' strokeLinejoin='round' />
      <path d='M8.9 11.9l2.1 2.1 4.2-4.2' fill='none' stroke='currentColor' strokeWidth='1.8'
        strokeLinecap='round' strokeLinejoin='round' />
    </svg>
  )
}

export function BoltIcon({ size = 15, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M13.2 2.5L5.4 13.4h5.2l-1.1 8.1 8.3-11.3h-5.2l.6-7.7z' fill='currentColor' />
    </svg>
  )
}

export function SendIcon({ size = 16, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M12 19.5V5M6 11l6-6 6 6' fill='none' stroke='currentColor' strokeWidth='2'
        strokeLinecap='round' strokeLinejoin='round' />
    </svg>
  )
}

export function ChevronIcon({ size = 11, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M6.5 9.2l5.5 5.4 5.5-5.4' fill='none' stroke='currentColor' strokeWidth='2'
        strokeLinecap='round' strokeLinejoin='round' />
    </svg>
  )
}

export function MiniChevronIcon({ size = 11, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M9.2 6.5l5.4 5.5-5.4 5.5' fill='none' stroke='currentColor' strokeWidth='2'
        strokeLinecap='round' strokeLinejoin='round' />
    </svg>
  )
}

export function CheckIcon({ size = 12, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M5 12.7l4.6 4.6L19 7.2' fill='none' stroke='currentColor' strokeWidth='2.2'
        strokeLinecap='round' strokeLinejoin='round' />
    </svg>
  )
}

export function LayersIcon({ size = 13, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <g fill='none' stroke='currentColor' strokeWidth='1.7' strokeLinecap='round' strokeLinejoin='round'>
        <path d='M12 3.6l8.4 4.3L12 12.2 3.6 7.9 12 3.6z' />
        <path d='M4.4 11.7L12 15.6l7.6-3.9M4.4 15.5L12 19.4l7.6-3.9' />
      </g>
    </svg>
  )
}

export function SlidersIcon({ size = 13, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <path d='M4.5 7.5h15M4.5 16.5h15' fill='none' stroke='currentColor' strokeWidth='1.8' strokeLinecap='round' />
      <circle cx='9' cy='7.5' r='2.1' fill='none' stroke='currentColor' strokeWidth='1.8' />
      <circle cx='15.5' cy='16.5' r='2.1' fill='none' stroke='currentColor' strokeWidth='1.8' />
    </svg>
  )
}

export function SearchIcon({ size = 13, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <circle cx='10.6' cy='10.6' r='6' fill='none' stroke='currentColor' strokeWidth='1.8' />
      <path d='M15.2 15.2l4.4 4.4' fill='none' stroke='currentColor' strokeWidth='1.8' strokeLinecap='round' />
    </svg>
  )
}

export function StopIcon({ size = 13, className }: IconProps) {
  return (
    <svg viewBox='0 0 24 24' width={size} height={size} className={className} aria-hidden='true'>
      <rect x='6.8' y='6.8' width='10.4' height='10.4' rx='2.4' fill='currentColor' />
    </svg>
  )
}
