import { ref } from 'vue';

import {
  deleteProductUnitApi,
  getProductUnitOptionsApi,
  saveProductUnitApi,
  updateProductUnitApi,
  type ProductUnitOption,
} from '#/api/core/product/product-unit';

const units = ref<ProductUnitOption[]>([]);
let loadPromise: null | Promise<void> = null;

async function loadUnits() {
  try {
    const list = await getProductUnitOptionsApi();
    units.value = Array.isArray(list) ? list : [];
  } catch {
    units.value = [];
  } finally {
    loadPromise = null;
  }
}

function precisionOf(name?: null | string): number {
  if (name) {
    const hit = units.value.find((item) => item.name === name);
    if (hit) return hit.decimalPlaces ?? 0;
  }
  return 2;
}

export function formatQty(value: unknown, unit?: null | string): string {
  if (value === null || value === undefined || value === '') return '-';
  const num = Number(value);
  if (!Number.isFinite(num)) return '-';
  let text = num.toFixed(precisionOf(unit));
  if (text.includes('.')) {
    text = text.replace(/0+$/, '').replace(/\.$/, '');
  }
  return text === '' ? '0' : text;
}

export function useProductUnits() {
  function ensureUnits() {
    if (units.value.length > 0 || loadPromise) return;
    loadPromise = loadUnits();
  }

  async function addUnit(name: string, decimalPlaces: number) {
    await saveProductUnitApi({ name, decimalPlaces });
    await loadUnits();
  }

  async function updateUnit(id: number | string, decimalPlaces: number) {
    await updateProductUnitApi({ decimalPlaces, id: Number(id) });
    await loadUnits();
  }

  async function removeUnit(id: number | string) {
    await deleteProductUnitApi(Number(id));
    await loadUnits();
  }

  return { addUnit, ensureUnits, precisionOf, removeUnit, units, updateUnit };
}
