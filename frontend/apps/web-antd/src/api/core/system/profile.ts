import { requestClient } from '#/api/request';

// ========== 类型 ==========
export interface VisibilityConfig {
  showMobile: boolean;
  showWechat: boolean;
  showSkills: boolean;
  showBirthday: boolean;
}

export interface ResumeItem {
  id?: number;
  kind: number; // 1教育 2工作 3证书
  title?: string;
  org?: string;
  startDate?: string;
  endDate?: string;
  remark?: string;
  isPublic?: number;
}

export interface EmergencyContactItem {
  id?: number;
  name: string;
  relation?: string;
  mobile: string;
  sort?: number;
}

export interface MyProfileVO {
  basic: {
    nickName?: string;
    gender?: number;
    email?: string;
    avatar?: string;
    intro?: string;
    mobileMasked?: string;
  };
  employ: {
    userName?: string;
    deptNames: string[];
    postNames: string[];
    directManagerId?: number;
    directManagerName?: string;
    hireDate?: string;
    probationMonths?: number;
  };
  idCard: { masked?: string; locked: boolean };
  bank: {
    maskedCardNo?: string;
    bankName?: string;
    maskedAccountName?: string;
    locked: boolean;
  };
  visibility: VisibilityConfig;
  resume: ResumeItem[];
  emergencyContacts: EmergencyContactItem[];
}

export interface CardVO {
  adminId: number;
  nickName?: string;
  avatar?: string;
  deptNames: string[];
  postNames: string[];
  directManagerName?: string;
  email?: string;
  intro?: string;
  mobile?: string;
  wechat?: string;
  skills: string[];
  birthday?: string;
  online: boolean;
}

export interface HrArchiveListVO {
  id: number;
  userName?: string;
  nickName?: string;
  deptNames: string[];
  postNames: string[];
  hireDate?: string;
  idLocked: boolean;
  bankLocked: boolean;
  completeness: number;
}

export interface HrArchiveDetailVO {
  id: number;
  userName?: string;
  nickName?: string;
  gender?: number;
  email?: string;
  mobile?: string;
  avatar?: string;
  deptIds: number[];
  deptNames: string[];
  postIds: number[];
  postNames: string[];
  directManagerId?: number;
  directManagerName?: string;
  hireDate?: string;
  probationMonths?: number;
  probationRatio?: number;
  idCardNo?: string;
  idLocked: boolean;
  bankCardNo?: string;
  bankName?: string;
  bankAccountName?: string;
  bankLocked: boolean;
  status?: number;
  resume: ResumeItem[];
  emergencyContacts: EmergencyContactItem[];
}

export interface ProfileLogVO {
  id: number;
  adminId: number;
  field?: string;
  oldValue?: string;
  newValue?: string;
  operateType?: number;
  operatorName?: string;
  createTime?: string;
  createDate?: string;
}

// ========== 本人档案 ==========
export const getMyProfileApi = async () =>
  requestClient.get<MyProfileVO>('/api/system/profile/my');

export const updateBasicApi = async (data: {
  nickName?: string;
  gender?: number;
  email?: string;
  intro?: string;
  visibility?: VisibilityConfig;
  wechat?: string;
  birthday?: string;
}) => requestClient.put('/api/system/profile/basic', data);

export const submitIdCardApi = async (idCardNo: string) =>
  requestClient.post('/api/system/profile/id-card', { idCardNo });

export const submitBankApi = async (data: {
  bankCardNo: string;
  bankName?: string;
  bankAccountName?: string;
}) => requestClient.post('/api/system/profile/bank', data);

// ========== 简历 ==========
export const getResumeApi = async () =>
  requestClient.get<ResumeItem[]>('/api/system/profile/resume');

export const saveResumeApi = async (data: ResumeItem) =>
  requestClient.post('/api/system/profile/resume', data);

export const updateResumeApi = async (data: ResumeItem) =>
  requestClient.put('/api/system/profile/resume', data);

export const deleteResumeApi = async (id: number) =>
  requestClient.delete(`/api/system/profile/resume/${id}`);

// ========== 紧急联系人 ==========
export const getEmergencyContactsApi = async () =>
  requestClient.get<EmergencyContactItem[]>(
    '/api/system/profile/emergency-contact',
  );

export const saveEmergencyContactApi = async (data: EmergencyContactItem) =>
  requestClient.post('/api/system/profile/emergency-contact', data);

export const updateEmergencyContactApi = async (data: EmergencyContactItem) =>
  requestClient.put('/api/system/profile/emergency-contact', data);

export const deleteEmergencyContactApi = async (id: number) =>
  requestClient.delete(`/api/system/profile/emergency-contact/${id}`);

// ========== 同事名片 ==========
export const getCardApi = async (adminId: number) =>
  requestClient.get<CardVO>(`/api/system/profile/card/${adminId}`);

// ========== 本人变更日志 ==========
export const getMyLogApi = async (params: {
  page: number;
  pageSize: number;
}) =>
  requestClient.get('/api/system/profile/log', {
    params: { page: params.page, page_size: params.pageSize },
  });

// ========== HR 档案管理 ==========
export const getHrArchiveListApi = async (params: {
  page: number;
  pageSize: number;
  keyword?: string;
  filled?: boolean;
}) =>
  requestClient.get('/api/system/hr-archive/list', {
    params: {
      page: params.page,
      page_size: params.pageSize,
      keyword: params.keyword,
      filled: params.filled,
    },
  });

export const getHrArchiveDetailApi = async (adminId: number) =>
  requestClient.get<HrArchiveDetailVO>(`/api/system/hr-archive/${adminId}`);

export const updateHrArchiveApi = async (adminId: number, data: object) =>
  requestClient.put(`/api/system/hr-archive/${adminId}`, data);

export const unlockHrArchiveApi = async (adminId: number, field: string) =>
  requestClient.post(`/api/system/hr-archive/${adminId}/unlock`, { field });

export const getHrArchiveLogApi = async (params: {
  page: number;
  pageSize: number;
  adminId?: number;
}) =>
  requestClient.get('/api/system/hr-archive/logs', {
    params: {
      page: params.page,
      page_size: params.pageSize,
      admin_id: params.adminId,
    },
  });
