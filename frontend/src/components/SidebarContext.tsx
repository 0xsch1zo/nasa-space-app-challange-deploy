import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import { useDisclosure } from '@mantine/hooks';

/**
 * Lets a page put its own navigation into the application's side bar.
 *
 * `AppLayout` owns the `AppShell.Navbar` and exposes its DOM slot here; a
 * page renders `<Sidebar>…</Sidebar>` and the content is portalled into the
 * slot. The navbar is shown only while some page provides content.
 */
interface SidebarContextValue {
  slot: HTMLDivElement | null;
  setSlot: (el: HTMLDivElement | null) => void;
  hasContent: boolean;
  setHasContent: (v: boolean) => void;
  /** Mobile drawer state. */
  opened: boolean;
  toggle: () => void;
  close: () => void;
}

const SidebarContext = createContext<SidebarContextValue | null>(null);

export function SidebarProvider({ children }: { children: ReactNode }) {
  const [slot, setSlot] = useState<HTMLDivElement | null>(null);
  const [hasContent, setHasContent] = useState(false);
  const [opened, { toggle, close }] = useDisclosure(false);
  const value = useMemo(
    () => ({ slot, setSlot, hasContent, setHasContent, opened, toggle, close }),
    [slot, hasContent, opened, toggle, close],
  );
  return <SidebarContext.Provider value={value}>{children}</SidebarContext.Provider>;
}

export function useSidebar() {
  const ctx = useContext(SidebarContext);
  if (!ctx) throw new Error('useSidebar must be used inside SidebarProvider');
  return ctx;
}

/** Renders its children inside the application side bar while mounted. */
export function Sidebar({ children }: { children: ReactNode }) {
  const { slot, setHasContent } = useSidebar();
  useEffect(() => {
    setHasContent(true);
    return () => setHasContent(false);
  }, [setHasContent]);
  return slot ? createPortal(children, slot) : null;
}
