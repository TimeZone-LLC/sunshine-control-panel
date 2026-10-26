<template>
  <div class="shortcut-reference">
    <section v-for="section in sections" :key="section.title">
      <h2>{{ section.title }}</h2>
      <template v-for="group in section.groups" :key="group.title">
        <h3>{{ group.title }}</h3>
        <dl>
          <div v-for="entry in group.entries" :key="entry.keys">
            <dt><kbd>{{ entry.keys }}</kbd></dt>
            <dd>{{ entry.action }}</dd>
          </div>
        </dl>
        <p v-if="group.note" class="note">{{ group.note }}</p>
      </template>
    </section>
    <a href="https://github.com/moonlight-stream/moonlight-docs/wiki/Setup-Guide#keyboardmousegamepad-input-options" target="_blank" rel="noreferrer">
      Moonlight input reference
    </a>
  </div>
</template>

<script setup>
const sections = [
  {
    title: 'PC client',
    groups: [
      {
        title: 'Session control',
        entries: [
          ['Ctrl+Alt+Shift+Q', 'Quit the stream without closing the game'],
          ['Ctrl+Alt+Shift+D', 'Minimise the streaming window'],
          ['Ctrl+Alt+Shift+X', 'Toggle fullscreen'],
          ['Ctrl+Alt+Shift+S', 'Toggle performance statistics'],
        ],
        note: 'Performance statistics are not available on Steam Link or Raspberry Pi.',
      },
      {
        title: 'Mouse and clipboard',
        entries: [
          ['Ctrl+Alt+Shift+Z', 'Toggle mouse and keyboard capture'],
          ['Ctrl+Alt+Shift+M', 'Toggle pointer capture and direct control'],
          ['Ctrl+Alt+Shift+C', 'Toggle the local cursor'],
          ['Ctrl+Alt+Shift+L', 'Lock the pointer to the video area'],
          ['Ctrl+Alt+Shift+V', 'Type clipboard text on the host'],
        ],
      },
    ],
  },
  {
    title: 'Android client',
    groups: [
      {
        title: 'Touch input',
        entries: [
          ['One-finger drag', 'Move the pointer'],
          ['One-finger tap', 'Left click'],
          ['One-finger long press', 'Right click'],
          ['Two-finger drag', 'Scroll or pan'],
          ['Three-finger tap', 'Show the on-screen keyboard'],
        ],
      },
      {
        title: 'Gamepad as mouse',
        entries: [
          ['Hold Start', 'Toggle mouse emulation'],
          ['Stick', 'Move the pointer'],
          ['A / B', 'Left / right click'],
          ['R1+Start', 'Act as Select'],
          ['Start+Select', 'Act as Mode'],
        ],
      },
    ],
  },
  {
    title: 'iOS and tvOS',
    groups: [
      {
        title: 'Gestures',
        entries: [
          ['Left-edge swipe', 'Disconnect the stream on iOS'],
          ['Three-finger tap', 'Show the on-screen keyboard'],
          ['Double press Menu', 'Disconnect on Apple TV'],
          ['Remote touchpad', 'Move the pointer and click'],
        ],
        note: 'iOS and tvOS 13 or later support Xbox One S and PS4 controllers over Bluetooth.',
      },
    ],
  },
].map((section) => ({
  ...section,
  groups: section.groups.map((group) => ({
    ...group,
    entries: group.entries.map(([keys, action]) => ({ keys, action })),
  })),
}))
</script>

<style scoped>
.shortcut-reference {
  max-width: 980px;
  padding: 16px;
  color: var(--text);
}

section + section { margin-top: 20px; }
h2 { margin: 0 0 8px; border-bottom: 1px solid var(--border); padding-bottom: 5px; font-size: 14px; }
h3 { margin: 12px 0 6px; color: var(--text-dim); font-size: 12px; font-weight: 500; }
dl { display: grid; grid-template-columns: repeat(auto-fit, minmax(330px, 1fr)); gap: 1px; margin: 0; }
dl > div { min-height: 28px; display: grid; grid-template-columns: minmax(150px, auto) 1fr; align-items: center; gap: 10px; background: var(--bg-panel); padding: 4px 8px; }
dt, dd { margin: 0; }
dd { color: var(--text-dim); }
kbd { border: 1px solid var(--border-strong); background: var(--bg-input); padding: 1px 5px; font-family: var(--font-mono); font-size: 11px; }
.note { margin: 6px 0 0; border-left: 2px solid var(--border-strong); padding: 5px 8px; color: var(--text-dim); font-size: 12px; }
a { display: inline-block; margin-top: 18px; color: var(--text); text-underline-offset: 2px; }
@media (max-width: 720px) { dl { grid-template-columns: 1fr; } }
</style>
