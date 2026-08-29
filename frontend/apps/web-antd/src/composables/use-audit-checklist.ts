import type { Ref } from 'vue';

import { computed } from 'vue';

import { useUserStore } from '@vben/stores';

export interface AuditChecklistItem {
  key: string;
  label: string;
  desc: string;
  tab: string;
  done: boolean;
}

export interface AuditStateInfo {
  type:
    | 'loading'
    | 'approved'
    | 'pending'
    | 'rejected'
    | 'withdrawn'
    | 'modify'
    | 'none';
  label: string;
  color: string;
}

export function useAuditChecklist(data: Ref<any>, profile: Ref<any>) {
  const userStore = useUserStore();

  const currentUserId = computed(() =>
    Number(userStore.userInfo?.userId ?? userStore.userInfo?.id ?? 0),
  );

  const approved = computed(() => data.value?.auditStatus === 1);
  const instances = computed<any[]>(() => data.value?.instances || []);
  const latest = computed<any>(() => instances.value.at(-1) || null);

  const auditState = computed<AuditStateInfo>(() => {
    if (!data.value) return { type: 'loading', label: '加载中', color: 'default' };
    if (approved.value) return { type: 'approved', label: '已通过', color: 'success' };
    const st = latest.value?.status;
    if (st === 1 || st === 2) return { type: 'pending', label: '审批中', color: 'processing' };
    if (st === 4) return { type: 'rejected', label: '已驳回', color: 'error' };
    if (st === 5) return { type: 'withdrawn', label: '已撤回', color: 'default' };
    if (st === 6) return { type: 'modify', label: '待修改', color: 'warning' };
    return { type: 'none', label: '未提交', color: 'default' };
  });

  const canSubmit = computed(() =>
    ['none', 'rejected', 'withdrawn', 'modify'].includes(auditState.value.type),
  );

  const checklist = computed<AuditChecklistItem[]>(() => {
    const p = profile.value || {};
    const basicDone = !!(p?.basic?.nickName && String(p.basic.nickName).trim());
    const resumeDone = (p?.resume?.length ?? 0) > 0;
    const financeDone = !!(p?.idCard?.masked && p?.bank?.maskedCardNo);
    const contactDone = (p?.emergencyContacts?.length ?? 0) > 0;
    return [
      {
        key: 'basic',
        label: '个人信息',
        desc: '昵称 / 姓名等基础信息',
        tab: 'basic',
        done: basicDone,
      },
      {
        key: 'resume',
        label: '个人简历',
        desc: '教育 / 工作经历至少一条',
        tab: 'resume',
        done: resumeDone,
      },
      {
        key: 'finance',
        label: '财务信息',
        desc: '身份证与工资卡',
        tab: 'idfinance',
        done: financeDone,
      },
      {
        key: 'contact',
        label: '紧急联系人',
        desc: '至少一位紧急联系人',
        tab: 'emergency',
        done: contactDone,
      },
    ];
  });

  const doneCount = computed(() => checklist.value.filter((i) => i.done).length);
  const allDone = computed(() => doneCount.value === checklist.value.length);

  const selfRow = computed(() => {
    const u: any = userStore.userInfo || {};
    const p: any = profile.value || {};
    const basic = p?.basic || {};
    const employ = p?.employ || {};
    return {
      id: currentUserId.value,
      nickName: basic.nickName || u.nickName || u.realName || '',
      userName: employ.userName || u.username || '',
      auditStatus: data.value?.auditStatus ?? 0,
      approvalStatus: latest.value?.status ?? undefined,
      approvalInstanceId: data.value?.latestInstanceId ?? undefined,
      flowCode: latest.value?.flowCode || 'hire_approval',
      deptName: (employ.deptNames || []).join('、'),
      postName: (employ.postNames || []).join('、'),
      roleName: u.roleName || '',
      mobile: basic.mobileMasked || '',
      email: basic.email || '',
      hireDate: employ.hireDate || '',
      directManagerName: employ.directManagerName || '',
    };
  });

  return {
    currentUserId,
    approved,
    instances,
    latest,
    auditState,
    canSubmit,
    checklist,
    doneCount,
    allDone,
    selfRow,
  };
}
