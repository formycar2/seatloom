import { useEffect, useState } from 'react';
import { breakpoints } from '../styles/theme';

export type ScreenMode = 'desktop' | 'compact' | 'tablet' | 'mobile';

interface ResponsiveState {
  mode: ScreenMode;
  isDesktop: boolean;
  isCompact: boolean;
  isTablet: boolean;
  isMobile: boolean;
  width: number;
}

export function useResponsive(): ResponsiveState {
  const [width, setWidth] = useState(
    typeof window !== 'undefined' ? window.innerWidth : 1440
  );

  useEffect(() => {
    const onResize = () => setWidth(window.innerWidth);
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  const isMobile = width < breakpoints.mobile;
  const isTablet = width >= breakpoints.mobile && width < breakpoints.tablet;
  const isCompact = width >= breakpoints.tablet && width < breakpoints.compact;
  const isDesktop = width >= breakpoints.compact;

  const mode: ScreenMode = isMobile
    ? 'mobile'
    : isTablet
    ? 'tablet'
    : isCompact
    ? 'compact'
    : 'desktop';

  return { mode, isDesktop, isCompact, isTablet, isMobile, width };
}
