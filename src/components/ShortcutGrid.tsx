import { ShortcutTile } from './ShortcutTile';
import type { DropshotConfig } from '../types/config';

interface Props {
  config: DropshotConfig;
}

/**
 * Renders the shortcuts container.
 *
 * The container is `position: relative` and sized to `application.menuSize`
 * so each `ShortcutTile` can be positioned absolutely using its `x`/`y`
 * coordinates from the config.
 */
export function ShortcutGrid({ config }: Props) {
  const { application, effects, shortcuts } = config;

  const containerStyle: Record<string, string> = {
    position: 'relative',
    // menuSize is guaranteed by the schema (has defaults in Rust / TypeScript).
    // We use a conditional to avoid the '100%px' bug if width is somehow absent.
    width:  application.menuSize ? `${application.menuSize.width}px`  : '100%',
    height: application.menuSize ? `${application.menuSize.height}px` : '100%',
    flexShrink: '0',
  };

  return (
    <div style={containerStyle}>
      {shortcuts.map((s) => (
        <ShortcutTile
          key={`${s.name}:${s.path}:${s.x}:${s.y}`}
          shortcut={s}
          effects={effects}
        />
      ))}
    </div>
  );
}
