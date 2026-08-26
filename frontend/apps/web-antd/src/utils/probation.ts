/**
 * 劳动合同法第十九条：试用期法定上限规则（与后端 hire_salary_service::max_probation_month 保持一致）
 * <3个月不得约定试用期；≥3个月且<1年 ≤1个月；≥1年且<3年 ≤2个月；≥3年固定期或无固定期限 ≤6个月
 */
export function calcMaxProbation(
  contractType?: number | null,
  contractMonths?: number | null,
): number {
  if (contractType === 2) return 6;
  const m = Number(contractMonths ?? 0);
  if (m >= 36) return 6;
  if (m >= 12) return 2;
  if (m >= 3) return 1;
  return 0;
}

/** 合同信息提示文案（用于审批填写区/表单帮助） */
export function formatContractText(
  contractType?: number | null,
  contractMonths?: number | null,
): string {
  if (contractType === 2) {
    return `无固定期限合同 · 试用期上限 ${calcMaxProbation(contractType, contractMonths)} 个月`;
  }
  const m = Number(contractMonths ?? 0);
  if (!m || calcMaxProbation(contractType, contractMonths) === 0) {
    return '合同期限未录入或不足 3 个月，依法不得约定试用期';
  }
  const label = m % 12 === 0 ? `${m / 12} 年` : `${m} 个月`;
  return `${label}固定期限合同 · 试用期上限 ${calcMaxProbation(contractType, contractMonths)} 个月`;
}
