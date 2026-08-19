<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';

import * as monaco from 'monaco-editor';
// worker 路径通过 vite.config.ts 的 alias 绕过 monaco-editor@0.56 exports 限制
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import CssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import HtmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import JsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import TsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';

const props = withDefaults(
  defineProps<{
    height?: string;
    language?: string;
    modelValue?: string;
    options?: Record<string, any>;
    readOnly?: boolean;
    theme?: string;
  }>(),
  {
    height: '400px',
    language: 'html',
    modelValue: '',
    options: () => ({}),
    readOnly: false,
    theme: 'vs',
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

// 配置 Monaco Web Worker（必须在创建编辑器实例之前设置）
const monacoGlobal = globalThis as typeof globalThis & {
  MonacoEnvironment?: monaco.Environment;
};
if (!monacoGlobal.MonacoEnvironment) {
  monacoGlobal.MonacoEnvironment = {
    getWorker(_workerId: string, label: string) {
      if (label === 'json') return new JsonWorker();
      if (label === 'css' || label === 'scss' || label === 'less')
        return new CssWorker();
      if (label === 'html' || label === 'handlebars' || label === 'razor')
        return new HtmlWorker();
      if (label === 'typescript' || label === 'javascript')
        return new TsWorker();
      return new EditorWorker();
    },
  };
}

const editorRef = ref<HTMLElement>();
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

onMounted(() => {
  if (!editorRef.value) return;
  editor = monaco.editor.create(editorRef.value, {
    value: props.modelValue,
    language: props.language,
    theme: props.theme,
    automaticLayout: true,
    fontSize: 14,
    wordWrap: 'on',
    tabSize: 2,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    readOnly: props.readOnly,
    ...props.options,
  });

  editor.onDidChangeModelContent(() => {
    emit('update:modelValue', editor?.getValue() || '');
  });
});

watch(
  () => props.modelValue,
  (newVal) => {
    if (editor && newVal !== editor.getValue()) {
      editor.setValue(newVal);
    }
  },
);

watch(
  () => props.language,
  (newLang) => {
    if (editor) {
      const model = editor.getModel();
      if (model) monaco.editor.setModelLanguage(model, newLang);
    }
  },
);

watch(
  () => props.theme,
  (newTheme) => {
    if (editor) monaco.editor.setTheme(newTheme);
  },
);

watch(
  () => props.readOnly,
  (val) => {
    editor?.updateOptions({ readOnly: val });
  },
);

onBeforeUnmount(() => {
  editor?.dispose();
});

// 在光标位置插入文本（若编辑器未聚焦则追加到末尾）
function insertText(text: string) {
  if (!editor) return;
  editor.focus();
  const selection = editor.getSelection();
  if (selection) {
    editor.executeEdits('insert-snippet', [
      {
        range: selection,
        text,
        forceMoveMarkers: true,
      },
    ]);
  } else {
    const model = editor.getModel();
    if (model) {
      const lastLine = model.getLineCount();
      const lastCol = model.getLineMaxColumn(lastLine);
      editor.executeEdits('insert-snippet', [
        {
          range: new monaco.Range(lastLine, lastCol, lastLine, lastCol),
          text: `\n${text}`,
          forceMoveMarkers: true,
        },
      ]);
    }
  }
}

defineExpose({ insertText });
</script>

<template>
  <div ref="editorRef" class="code-editor" :style="{ height }"></div>
</template>

<style scoped>
.code-editor {
  width: 100%;
  overflow: hidden;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
}
</style>
