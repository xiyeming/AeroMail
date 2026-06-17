import { computed } from 'vue';
import { useWindowSize } from '@vueuse/core';

export type LayoutMode = 'mobile' | 'compact' | 'tablet' | 'desktop' | 'wide';

export function useResponsive() {
  const { width } = useWindowSize();

  const layoutMode = computed<LayoutMode>(() => {
    const w = width.value;
    if (w < 800) return 'mobile';
    if (w < 1140) return 'compact';
    if (w < 1400) return 'tablet';
    if (w < 1920) return 'desktop';
    return 'wide';
  });

  const isWideScreen = computed(() => width.value >= 1920);
  const isCollapsed = computed(() => width.value < 1140);

  return {
    width,
    layoutMode,
    isWideScreen,
    isCollapsed,
  };
}
