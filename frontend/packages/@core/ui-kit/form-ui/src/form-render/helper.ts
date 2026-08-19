import type {
  AnyZodObject,
  ZodDefault,
  ZodEffects,
  ZodNumber,
  ZodString,
  ZodTypeAny,
} from 'zod';

import { isObject, isString } from '@vben-core/shared/utils';

/**
 * Get the lowest level Zod type.
 * This will unpack optionals, refinements, etc.
 */
export function getBaseRules<
  ChildType extends AnyZodObject | ZodTypeAny = ZodTypeAny,
>(schema: ChildType | ZodEffects<ChildType>): ChildType | null {
  if (!schema || isString(schema)) return null;
  const def = (schema as any)?._def;
  if (!def) return null;
  if ('innerType' in def) return getBaseRules(def.innerType as ChildType);

  if ('schema' in def) return getBaseRules(def.schema as ChildType);

  return schema as ChildType;
}

/**
 * Search for a "ZodDefault" in the Zod stack and return its value.
 */
export function getDefaultValueInZodStack(schema: ZodTypeAny): any {
  if (!schema || isString(schema)) {
    return;
  }
  const typedSchema = schema as unknown as ZodDefault<ZodNumber | ZodString>;
  const def = (typedSchema as any)?._def;
  if (!def) return undefined;

  if (def.typeName === 'ZodDefault') return def.defaultValue();

  if ('innerType' in def) {
    return getDefaultValueInZodStack(def.innerType as unknown as ZodTypeAny);
  }
  if ('schema' in def) {
    return getDefaultValueInZodStack((def as any).schema as ZodTypeAny);
  }

  return undefined;
}

export function isEventObjectLike(obj: any) {
  if (!obj || !isObject(obj)) {
    return false;
  }
  return Reflect.has(obj, 'target') && Reflect.has(obj, 'stopPropagation');
}
