/**
 * Brand logos for each managed tool, used in the icon-only tab bar.
 * Real PNG marks for Claude / Cursor / Antigravity; Windsurf falls back to a
 * stylized teal sail until an official asset is added.
 */
import antigravityLogo from '@/assets/images/logos/agy.png'
import claudeLogo from '@/assets/images/logos/claude.png'
import cursorLogo from '@/assets/images/logos/cursor.png'
import windsurfLogo from '@/assets/images/logos/windsurf.png'

interface LogoProps {
  className?: string
}

const PNG_LOGOS: Record<string, string> = {
  claude: claudeLogo,
  'claude-code': claudeLogo,
  cursor: cursorLogo,
  antigravity: antigravityLogo,
  windsurf: windsurfLogo,
}

/** Fallback — generic monitor outline. */
function GenericLogo({ className }: LogoProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className}
      aria-hidden="true"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinejoin="round"
    >
      <rect
        x="3"
        y="4"
        width="18"
        height="12"
        rx="1.5"
      />
      <path d="M8 20 H16 M12 16 V20" />
    </svg>
  )
}

/** Render the brand logo for a tool key (claude / cursor / windsurf / antigravity). */
export function IdeLogo({
  name,
  className,
}: {
  name: string
  className?: string
}) {
  const src = PNG_LOGOS[name]
  if (src) {
    return (
      <img
        src={src}
        alt=""
        aria-hidden="true"
        className={`${className ?? ''} object-contain`}
      />
    )
  }
  return <GenericLogo className={className} />
}
