<script lang="ts">
  type Name =
    | 'chevron'
    | 'edit'
    | 'up'
    | 'down'
    | 'plus'
    | 'minus'
    | 'sort'
    | 'check'
    | 'back'
    | 'trash'
    | 'external'
    | 'settings'
    | 'image'
    | 'close'
    | 'folder'
    | 'swap'
    | 'search'
    | 'sortlines';

  let { name, size = 22 }: { name: Name; size?: number } = $props();

  const paths: Record<Name, string> = {
    chevron: 'M9 6l6 6-6 6',
    edit: 'M4 20h4L18.5 9.5l-4-4L4 16v4zM13 7l4 4',
    up: 'M6 15l6-6 6 6',
    down: 'M6 9l6 6 6-6',
    plus: 'M12 5v14M5 12h14',
    minus: 'M5 12h14',
    sort: 'M8 5v14M8 5L5 8M8 5l3 3M16 19V5M16 19l-3-3M16 19l3-3',
    check: 'M5 13l4 4L19 7',
    back: 'M15 6l-6 6 6 6',
    trash: 'M5 7h14M10 7V5h4v2M7 7l1 12h8l1-12',
    external: 'M14 5h5v5M19 5l-8 8M18 14v5H5V6h5',
    settings: 'M4 7h9M17 7h3M4 17h3M11 17h9M15 4v6M9 14v6',
    image: 'M4 5h16v14H4zM4 16l5-5 4 4 3-3 4 4M9 9.5h.01',
    close: 'M6 6l12 12M18 6L6 18',
    folder: 'M4 6h6l2 2h8v10H4z',
    swap: 'M7 7h11M15 4l3 3-3 3M17 17H6M9 14l-3 3 3 3',
    search: 'M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM20 20l-4-4',
    sortlines: 'M4 7h16M7 12h10M10 17h4',
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={paths[name]} />
</svg>
