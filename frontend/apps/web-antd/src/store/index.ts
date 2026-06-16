import { $t } from '@vben/locales';
import { computed } from 'vue';

export * from './auth';
export * from './user';

export const statusList = computed(() => [
  { value: 1, label: $t('enum.status.ON') },
  { value: 2, label: $t('enum.status.OFF') },
]);

export const methodList = computed(() => [
  { value: 'GET', label: 'GET' },
  { value: 'POST', label: 'POST' },
  { value: 'PUT', label: 'PUT' },
  { value: 'DELETE', label: 'DELETE' },
]);

export interface Timestamp {
  seconds: string;
  nanos: number;
}

export enum MenuType {
  FOLDER = 'FOLDER',
  MENU = 'MENU',
  BUTTON = 'BUTTON',
}
