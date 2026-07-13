import { addCollection } from '@vben/icons';

async function registerLucideIcons(): Promise<void> {
  try {
    const lucideIcons = await import('@iconify/json/json/lucide.json');
    const data = lucideIcons.default || lucideIcons;
    addCollection(data);
  } catch (error) {
    console.warn('Failed to preload lucide icons:', error);
  }
}

export { registerLucideIcons };
