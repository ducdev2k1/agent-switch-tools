/**
 * Brand-mark SVGs for each managed tool, used in the icon-only tab bar.
 * Stylized approximations (brand colour + signature shape) — not official
 * trademark assets — chosen so each tab is instantly distinguishable.
 */

interface LogoProps {
  className?: string
}

/** Anthropic / Claude — clay-orange radial burst. */
function ClaudeLogo({ className }: LogoProps) {
  const rays = [0, 30, 60, 90, 120, 150]
  return (
    <svg
      viewBox="0 0 24 24"
      className={className}
      aria-hidden="true"
    >
      <g
        stroke="#D97757"
        strokeWidth="2.3"
        strokeLinecap="round"
      >
        {rays.map((deg) => (
          <line
            key={deg}
            x1="12"
            y1="4"
            x2="12"
            y2="20"
            transform={`rotate(${deg} 12 12)`}
          />
        ))}
      </g>
    </svg>
  )
}

/** Cursor — monochrome 3D cube (adapts to theme via currentColor). */
function CursorLogo({ className }: LogoProps) {
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
      <polygon points="12,2 21,7 21,17 12,22 3,17 3,7" />
      <path d="M12 2 L12 12 M12 12 L21 7 M12 12 L3 7" />
    </svg>
  )
}

/** Windsurf — teal sail. */
function WindsurfLogo({ className }: LogoProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className}
      aria-hidden="true"
    >
      <path
        d="M13 3 L13 19 L4 19 Z"
        fill="#21C2A4"
      />
      <line
        x1="13"
        y1="3"
        x2="13"
        y2="21"
        stroke="#0E7C6B"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  )
}

/** Antigravity — blue orbit. */
function AntigravityLogo({ className }: LogoProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className}
      aria-hidden="true"
    >
      <circle
        cx="12"
        cy="12"
        r="3.4"
        fill="#4285F4"
      />
      <ellipse
        cx="12"
        cy="12"
        rx="10"
        ry="4"
        fill="none"
        stroke="#4285F4"
        strokeWidth="1.6"
        transform="rotate(-28 12 12)"
      />
    </svg>
  )
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
export function IdeLogo({ name, className }: { name: string; className?: string }) {
  switch (name) {
    case 'claude':
    case 'claude-code':
      return <ClaudeLogo className={className} />
    case 'cursor':
      return <CursorLogo className={className} />
    case 'windsurf':
      return <WindsurfLogo className={className} />
    case 'antigravity':
      return <AntigravityLogo className={className} />
    default:
      return <GenericLogo className={className} />
  }
}
