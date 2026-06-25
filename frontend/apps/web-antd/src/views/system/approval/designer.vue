<script lang="ts" setup>
import { computed, defineComponent, h, markRaw, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { VueFlow, useVueFlow, Handle, Position } from '@vue-flow/core';
import '@vue-flow/core/dist/style.css';
import '@vue-flow/core/dist/theme-default.css';
import { ArrowLeft, Save } from 'lucide-vue-next';
import { Button, Input, Select, Switch, message } from 'ant-design-vue';

import { getApprovalFlowDetailApi, saveApprovalFlowApi } from '#/api';

const route = useRoute();
const router = useRouter();

// ============ Node type config ============
const nodeStyleConfig: Record<
  number,
  { bg: string; shape: string; defaultLabel: string }
> = {
  1: { bg: '#52c41a', shape: 'circle', defaultLabel: '开始' },
  2: { bg: '#1890ff', shape: 'rect', defaultLabel: '审批' },
  3: { bg: '#fa8c16', shape: 'diamond', defaultLabel: '条件' },
  4: { bg: '#8c8c8c', shape: 'circle', defaultLabel: '结束' },
};

const nodePalette = [
  { type: 1, label: '开始节点', ...nodeStyleConfig[1] },
  { type: 2, label: '审批节点', ...nodeStyleConfig[2] },
  { type: 3, label: '条件分支', ...nodeStyleConfig[3] },
  { type: 4, label: '结束节点', ...nodeStyleConfig[4] },
];

const businessTypeOptions = [
  { value: 'contract', label: '合同' },
  { value: 'order', label: '订单' },
  { value: 'purchase', label: '采购' },
  { value: 'payment', label: '付款' },
  { value: 'expense', label: '报销' },
  { value: 'leave', label: '请假' },
];

const approverTypeOptions = [
  { value: 1, label: '指定用户' },
  { value: 2, label: '指定角色' },
  { value: 3, label: '部门主管' },
  { value: 4, label: '发起人自己' },
];

// ============ Custom node component ============
const FlowNode = defineComponent({
  name: 'FlowNode',
  props: {
    id: { type: String, default: '' },
    data: { type: Object, default: () => ({}) },
    selected: { type: Boolean, default: false },
  },
  setup(props) {
    return () => {
      const nodeType = props.data?.nodeType ?? 2;
      const cfg = nodeStyleConfig[nodeType] ?? nodeStyleConfig[2];
      const label = props.data?.nodeName || cfg.defaultLabel;
      const classes = [
        'flow-node',
        `shape-${cfg.shape}`,
        { selected: props.selected },
      ];
      const children: any[] = [
        h(Handle, {
          type: 'target',
          position: Position.Top,
          class: 'flow-handle',
        }),
      ];
      if (cfg.shape === 'diamond') {
        children.push(
          h('div', {
            class: 'diamond-shape',
            style: { background: cfg.bg },
          }),
        );
      }
      children.push(h('span', { class: 'flow-node-label' }, label));
      children.push(
        h(Handle, {
          type: 'source',
          position: Position.Bottom,
          class: 'flow-handle',
        }),
      );
      const style =
        cfg.shape === 'diamond'
          ? { background: 'transparent' }
          : { background: cfg.bg };
      return h('div', { class: classes, style }, children);
    };
  },
});

const nodeTypes = markRaw({ custom: FlowNode });

// ============ State ============
const flowId = ref<null | number>(null);
const flowCode = ref('');
const flowName = ref('');
const businessType = ref('contract');
const nodes = ref<any[]>([]);
const edges = ref<any[]>([]);
const selectedNodeId = ref<null | string>(null);
const selectedEdgeId = ref<null | string>(null);
const saving = ref(false);

let nodeSeq = 0;
function genNodeId() {
  nodeSeq += 1;
  return `n_${Date.now()}_${nodeSeq}`;
}

const {
  onConnect,
  onNodeClick,
  onEdgeClick,
  onPaneClick,
  screenToFlowCoordinate,
  fitView,
  zoomIn,
  zoomOut,
  toObject,
} = useVueFlow();

const selectedNode = computed<any>(() =>
  nodes.value.find((n) => n.id === selectedNodeId.value),
);

const selectedEdge = computed<any>(() =>
  edges.value.find((e) => e.id === selectedEdgeId.value),
);

const conditionEdges = computed<any[]>(() => {
  if (!selectedNode.value || selectedNode.value.data?.nodeType !== 3) return [];
  const id = selectedNode.value.id;
  return edges.value.filter((e) => e.source === id);
});

// ============ Drag and drop ============
function onDragStart(event: DragEvent, nodeType: number) {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/vueflow', String(nodeType));
    event.dataTransfer.effectAllowed = 'move';
  }
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
}

function onDrop(event: DragEvent) {
  const raw = event.dataTransfer?.getData('application/vueflow');
  if (!raw) return;
  const nodeType = Number(raw);
  const position = screenToFlowCoordinate({
    x: event.clientX,
    y: event.clientY,
  });
  const cfg = nodeStyleConfig[nodeType] ?? nodeStyleConfig[2];
  const node = {
    id: genNodeId(),
    type: 'custom',
    position,
    data: {
      nodeKey: '',
      nodeType,
      nodeName: cfg.defaultLabel,
      approverType: null,
      approverId: null,
      isFinal: false,
    },
  };
  nodes.value = [...nodes.value, node];
  selectedNodeId.value = node.id;
  selectedEdgeId.value = null;
}

// ============ Connections & selection ============
onConnect((connection) => {
  const edge = {
    id: `e_${Date.now()}_${Math.floor(Math.random() * 1000)}`,
    source: connection.source,
    target: connection.target,
    sourceHandle: connection.sourceHandle,
    targetHandle: connection.targetHandle,
    label: '',
    type: 'smoothstep',
    data: { conditionExpr: '', label: '' },
  };
  edges.value = [...edges.value, edge];
});

onNodeClick(({ node }) => {
  selectedNodeId.value = node.id;
  selectedEdgeId.value = null;
});

onEdgeClick(({ edge }) => {
  selectedEdgeId.value = edge.id;
  selectedNodeId.value = null;
});

onPaneClick(() => {
  selectedNodeId.value = null;
  selectedEdgeId.value = null;
});

function deleteSelectedNode() {
  if (!selectedNode.value) return;
  const id = selectedNode.value.id;
  nodes.value = nodes.value.filter((n) => n.id !== id);
  edges.value = edges.value.filter((e) => e.source !== id && e.target !== id);
  selectedNodeId.value = null;
}

function deleteSelectedEdge() {
  if (!selectedEdge.value) return;
  const id = selectedEdge.value.id;
  edges.value = edges.value.filter((e) => e.id !== id);
  selectedEdgeId.value = null;
}

// ============ Palette dot style ============
function paletteDotStyle(item: any) {
  return {
    background: item.bg,
    borderRadius: item.shape === 'circle' ? '50%' : '3px',
    transform: item.shape === 'diamond' ? 'rotate(45deg)' : 'none',
  };
}

// ============ Mini map ============
function miniNodeStyle(n: any) {
  const MINI_W = 140;
  const MINI_H = 90;
  if (!nodes.value.length) return { display: 'none' };
  const xs = nodes.value.map((nd) => nd.position.x);
  const ys = nodes.value.map((nd) => nd.position.y);
  const minX = Math.min(...xs);
  const minY = Math.min(...ys);
  const maxX = Math.max(...xs) + 140;
  const maxY = Math.max(...ys) + 80;
  const w = Math.max(1, maxX - minX);
  const h = Math.max(1, maxY - minY);
  const scale = Math.min(MINI_W / w, MINI_H / h);
  const cfg = nodeStyleConfig[n.data?.nodeType ?? 2] ?? nodeStyleConfig[2];
  return {
    left: `${(n.position.x - minX) * scale}px`,
    top: `${(n.position.y - minY) * scale}px`,
    width: `${Math.max(5, 40 * scale)}px`,
    height: `${Math.max(4, 24 * scale)}px`,
    background: cfg.bg,
    borderRadius: cfg.shape === 'circle' ? '50%' : '3px',
  };
}

// ============ Load ============
async function loadFlow(id: number) {
  try {
    const res: any = await getApprovalFlowDetailApi(id);
    const flow = res?.data?.data ?? res?.data ?? res ?? {};
    flowId.value = flow.id ?? id;
    flowCode.value = flow.flowCode ?? flow.flow_code ?? '';
    flowName.value = flow.flowName ?? flow.flow_name ?? '';
    businessType.value = flow.businessType ?? flow.business_type ?? 'contract';
    const rawNodes = flow.nodes ?? [];
    const rawEdges = flow.edges ?? [];
    nodes.value = rawNodes.map((n: any) => {
      const pos =
        n.position ?? {
          x: n.positionX ?? n.position_x ?? 0,
          y: n.positionY ?? n.position_y ?? 0,
        };
      return {
        id: n.nodeKey || n.node_key || String(n.id ?? genNodeId()),
        type: 'custom',
        position: { x: pos.x ?? 0, y: pos.y ?? 0 },
        data: {
          nodeKey: n.nodeKey ?? n.node_key ?? n.id ?? '',
          nodeType: n.nodeType ?? n.node_type ?? 2,
          nodeName: n.nodeName ?? n.node_name ?? '',
          approverType: n.approverType ?? n.approver_type ?? null,
          approverId: n.approverId ?? n.approver_id ?? null,
          isFinal: n.isFinal ?? n.is_final ?? false,
        },
      };
    });
    edges.value = rawEdges.map((e: any) => ({
      id: e.id ?? `e_${Date.now()}_${Math.floor(Math.random() * 1000)}`,
      source: e.source ?? e.sourceNodeKey ?? '',
      target: e.target ?? e.targetNodeKey ?? '',
      label: e.label ?? '',
      type: 'smoothstep',
      data: {
        conditionExpr:
          e.data?.conditionExpr ?? e.conditionExpr ?? e.condition_expr ?? '',
        label: e.data?.label ?? e.label ?? '',
      },
    }));
    setTimeout(() => fitView({ padding: 0.2 }), 120);
  } catch (e: any) {
    message.error(e?.message || '加载流程失败');
  }
}

// ============ Save ============
async function handleSave() {
  if (!flowName.value) {
    message.warning('请输入流程名称');
    return;
  }
  saving.value = true;
  try {
    const state = toObject();
    const sourceNodes = state?.nodes?.length ? state.nodes : nodes.value;
    const sourceEdges = state?.edges?.length ? state.edges : edges.value;
    const payload = {
      flowId: flowId.value,
      flowCode: flowCode.value,
      flowName: flowName.value,
      businessType: businessType.value,
      nodes: sourceNodes.map((n: any) => ({
        nodeKey: n.data?.nodeKey || n.id,
        nodeType: n.data?.nodeType ?? 2,
        nodeName: n.data?.nodeName ?? '',
        approverType: n.data?.approverType ?? null,
        approverId: n.data?.approverId ?? null,
        isFinal: n.data?.isFinal ?? false,
        positionX: n.position?.x ?? 0,
        positionY: n.position?.y ?? 0,
      })),
      edges: sourceEdges.map((e: any) => ({
        id: e.id,
        source: e.source,
        target: e.target,
        label: e.data?.label ?? e.label ?? '',
        conditionExpr: e.data?.conditionExpr ?? '',
      })),
    };
    await saveApprovalFlowApi(payload);
    message.success('保存成功');
    router.push('/system/approval');
  } catch (e: any) {
    message.error(e?.message || '保存失败');
  } finally {
    saving.value = false;
  }
}

function goBack() {
  router.push('/system/approval');
}

onMounted(() => {
  const id = route.query.id;
  if (id) {
    void loadFlow(Number(id));
  } else {
    nodes.value = [
      {
        id: genNodeId(),
        type: 'custom',
        position: { x: 250, y: 40 },
        data: {
          nodeKey: 'start',
          nodeType: 1,
          nodeName: '开始',
          approverType: null,
          approverId: null,
          isFinal: false,
        },
      },
      {
        id: genNodeId(),
        type: 'custom',
        position: { x: 250, y: 200 },
        data: {
          nodeKey: 'end',
          nodeType: 4,
          nodeName: '结束',
          approverType: null,
          approverId: null,
          isFinal: false,
        },
      },
    ];
  }
});
</script>

<template>
  <div class="flow-designer">
    <div class="flow-topbar">
      <div class="topbar-left">
        <Button :icon="h(ArrowLeft)" @click="goBack">返回</Button>
        <span class="topbar-title">审批流设计器</span>
      </div>
      <div class="topbar-center">
        <Input
          v-model:value="flowName"
          placeholder="流程名称"
          style="width: 200px"
        />
        <Input
          v-model:value="flowCode"
          placeholder="流程编码"
          style="width: 180px"
        />
        <Select
          v-model:value="businessType"
          :options="businessTypeOptions"
          placeholder="业务类型"
          style="width: 140px"
        />
      </div>
      <div class="topbar-right">
        <Button type="primary" :loading="saving" :icon="h(Save)" @click="handleSave">
          保存
        </Button>
      </div>
    </div>

    <div class="flow-body">
      <!-- Left: node palette -->
      <div class="flow-palette">
        <div class="palette-title">节点类型</div>
        <div
          v-for="item in nodePalette"
          :key="item.type"
          class="palette-item"
          draggable="true"
          @dragstart="onDragStart($event, item.type)"
        >
          <span class="palette-dot" :style="paletteDotStyle(item)"></span>
          <span>{{ item.label }}</span>
        </div>
        <div class="palette-hint">拖拽节点到画布</div>
      </div>

      <!-- Center: canvas -->
      <div class="flow-canvas" @drop="onDrop" @dragover="onDragOver">
        <VueFlow
          :nodes="nodes"
          :edges="edges"
          :node-types="nodeTypes"
          :default-edge-options="{ type: 'smoothstep' }"
          :fit-view-on-init="true"
          class="flow"
        />

        <div class="flow-controls">
          <Button size="small" @click="zoomIn()">+</Button>
          <Button size="small" @click="zoomOut()">−</Button>
          <Button size="small" @click="fitView()">适应</Button>
        </div>

        <div class="flow-minimap">
          <div class="minimap-title">缩略图</div>
          <div class="minimap-canvas">
            <div
              v-for="n in nodes"
              :key="n.id"
              class="mini-node"
              :style="miniNodeStyle(n)"
            ></div>
          </div>
        </div>
      </div>

      <!-- Right: properties -->
      <div class="flow-props">
        <div class="props-title">属性配置</div>

        <template v-if="selectedNode">
          <div class="props-section">
            <div class="props-field">
              <label>节点标识</label>
              <Input
                v-model:value="selectedNode.data.nodeKey"
                placeholder="如 start / approve1"
              />
            </div>
            <div class="props-field">
              <label>节点名称</label>
              <Input
                v-model:value="selectedNode.data.nodeName"
                placeholder="节点名称"
              />
            </div>

            <template v-if="selectedNode.data.nodeType === 2">
              <div class="props-field">
                <label>审批人类型</label>
                <Select
                  v-model:value="selectedNode.data.approverType"
                  :options="approverTypeOptions"
                  placeholder="选择类型"
                />
              </div>
              <div class="props-field">
                <label>审批人ID</label>
                <Input
                  v-model:value="selectedNode.data.approverId"
                  placeholder="审批人ID"
                />
              </div>
              <div class="props-field inline">
                <label>是否终审</label>
                <Switch v-model:checked="selectedNode.data.isFinal" />
              </div>
            </template>

            <div class="props-actions">
              <Button danger size="small" @click="deleteSelectedNode">
                删除节点
              </Button>
            </div>
          </div>

          <template v-if="selectedNode.data.nodeType === 3">
            <div class="props-section">
              <div class="props-section-title">出边条件</div>
              <div v-if="conditionEdges.length === 0" class="props-empty">
                暂无出边，请从条件节点拖拽连线
              </div>
              <div v-for="e in conditionEdges" :key="e.id" class="edge-cond">
                <div class="edge-cond-head">→ {{ e.target || '未连接' }}</div>
                <Input
                  v-model:value="e.data.label"
                  placeholder="分支标签"
                  size="small"
                />
                <Input
                  v-model:value="e.data.conditionExpr"
                  placeholder="条件表达式，如 amount > 10000"
                  size="small"
                />
              </div>
            </div>
          </template>
        </template>

        <template v-else-if="selectedEdge">
          <div class="props-section">
            <div class="props-field">
              <label>分支标签</label>
              <Input
                v-model:value="selectedEdge.data.label"
                placeholder="显示在连线上的标签"
              />
            </div>
            <div class="props-field">
              <label>条件表达式</label>
              <Input
                v-model:value="selectedEdge.data.conditionExpr"
                placeholder="如 amount > 10000"
              />
            </div>
            <div class="props-actions">
              <Button danger size="small" @click="deleteSelectedEdge">
                删除连线
              </Button>
            </div>
          </div>
        </template>

        <div v-else class="props-empty-hint">请选择节点或连线进行配置</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.flow-designer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f5f5f5;
}

.flow-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 16px;
  background: #fff;
  border-bottom: 1px solid #e8e8e8;
}

.topbar-left,
.topbar-center {
  display: flex;
  align-items: center;
  gap: 10px;
}

.topbar-title {
  font-size: 15px;
  font-weight: 600;
}

.flow-body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.flow-palette {
  width: 180px;
  padding: 12px;
  background: #fff;
  border-right: 1px solid #e8e8e8;
}

.palette-title {
  margin-bottom: 12px;
  color: #333;
  font-weight: 600;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
  padding: 10px 12px;
  font-size: 13px;
  background: #fafafa;
  border: 1px dashed #d9d9d9;
  border-radius: 6px;
  cursor: grab;
  user-select: none;
}

.palette-item:hover {
  background: #e6f7ff;
  border-color: #1890ff;
}

.palette-dot {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  display: inline-block;
}

.palette-hint {
  margin-top: 8px;
  color: #999;
  font-size: 12px;
}

.flow-canvas {
  position: relative;
  flex: 1;
  min-width: 0;
}

.flow {
  width: 100%;
  height: 100%;
  background-color: #fff;
  background-image: radial-gradient(#e5e5e5 1px, transparent 1px);
  background-size: 16px 16px;
}

.flow-controls {
  position: absolute;
  bottom: 12px;
  left: 12px;
  z-index: 10;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.flow-controls :deep(.ant-btn) {
  width: 36px;
}

.flow-minimap {
  position: absolute;
  right: 12px;
  bottom: 12px;
  z-index: 10;
  width: 160px;
  padding: 8px;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
}

.minimap-title {
  margin-bottom: 6px;
  color: #666;
  font-size: 12px;
}

.minimap-canvas {
  position: relative;
  width: 140px;
  height: 90px;
  overflow: hidden;
  background: #fafafa;
  border-radius: 4px;
}

.mini-node {
  position: absolute;
  opacity: 0.85;
}

.flow-props {
  width: 300px;
  padding: 12px;
  overflow-y: auto;
  background: #fff;
  border-left: 1px solid #e8e8e8;
}

.props-title {
  margin-bottom: 12px;
  color: #333;
  font-weight: 600;
}

.props-section {
  margin-bottom: 16px;
}

.props-section-title {
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
}

.props-field {
  margin-bottom: 10px;
}

.props-field.inline {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.props-field label {
  display: block;
  margin-bottom: 4px;
  color: #666;
  font-size: 12px;
}

.props-field.inline label {
  margin-bottom: 0;
}

.props-actions {
  margin-top: 8px;
}

.props-empty,
.props-empty-hint {
  padding: 20px 0;
  color: #999;
  font-size: 13px;
  text-align: center;
}

.edge-cond {
  margin-bottom: 10px;
  padding: 8px;
  background: #fafafa;
  border-radius: 4px;
}

.edge-cond-head {
  margin-bottom: 6px;
  color: #666;
  font-size: 12px;
}

.edge-cond :deep(.ant-input) {
  margin-bottom: 4px;
}
</style>

<style>
.flow-node {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 12px;
  font-weight: 500;
  position: relative;
  border: 2px solid transparent;
  transition:
    box-shadow 0.15s,
    filter 0.15s;
}

.flow-node.shape-circle {
  width: 80px;
  height: 80px;
  border-radius: 50%;
}

.flow-node.shape-rect {
  width: 140px;
  height: 50px;
  border-radius: 6px;
}

.flow-node.shape-diamond {
  width: 90px;
  height: 90px;
  background: transparent !important;
}

.flow-node.shape-diamond .diamond-shape {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 64px;
  height: 64px;
  transform: translate(-50%, -50%) rotate(45deg);
  border-radius: 4px;
}

.flow-node-label {
  position: relative;
  z-index: 1;
  padding: 0 6px;
  line-height: 1.3;
  text-align: center;
  word-break: break-all;
  pointer-events: none;
}

.flow-node.selected {
  box-shadow:
    0 0 0 3px #fff,
    0 0 0 5px #1890ff;
}

.flow-node.shape-diamond.selected {
  box-shadow: none;
}

.flow-node.shape-diamond.selected .diamond-shape {
  box-shadow: 0 0 0 3px #1890ff;
}

.flow-handle {
  width: 8px !important;
  height: 8px !important;
  background: #1890ff;
  border: 1px solid #fff;
}
</style>
