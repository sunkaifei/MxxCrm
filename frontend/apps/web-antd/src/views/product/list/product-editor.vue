<script lang="ts">
import { defineComponent, ref, onMounted, onBeforeUnmount, nextTick, watch } from 'vue';

// 从 CDN 加载 wangeditor（绕过 Vite optimizer 的兼容性问题）
const WANG_EDITOR_CDN = 'https://unpkg.com/@wangeditor/editor@5.1.23/dist/index.js';
const WANG_EDITOR_CSS_CDN = 'https://unpkg.com/@wangeditor/editor@5.1.23/dist/css/style.css';

let wangEditorModule: any = null;

async function loadWangEditor(): Promise<any> {
  if (wangEditorModule) return wangEditorModule;

  // 加载 CSS
  if (!document.querySelector('link[data-wangeditor-css]')) {
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = WANG_EDITOR_CSS_CDN;
    link.setAttribute('data-wangeditor-css', '');
    document.head.appendChild(link);
  }

  // 加载 JS
  await new Promise<void>((resolve, reject) => {
    if ((window as any).wangEditor) {
      resolve();
      return;
    }
    const script = document.createElement('script');
    script.src = WANG_EDITOR_CDN;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('Failed to load wangEditor from CDN'));
    document.head.appendChild(script);
  });

  wangEditorModule = (window as any).wangEditor;
  return wangEditorModule;
}

export default defineComponent({
  name: 'ProductEditor',
  props: {
    modelValue: { type: String, default: '' },
  },
  emits: ['update:modelValue'],
  setup(props, { emit }) {
    const editorRef = ref<any>(null);
    const editorContainerRef = ref<HTMLElement | null>(null);
    const toolbarContainerRef = ref<HTMLElement | null>(null);

    async function initEditor() {
      const WE = await loadWangEditor();
      if (!WE) return;

      await nextTick();
      if (!editorContainerRef.value || !toolbarContainerRef.value) return;

      if (editorRef.value) {
        editorRef.value.destroy();
        editorRef.value = null;
      }

      const editor = WE.createEditor({
        selector: editorContainerRef.value,
        config: {
          placeholder: '请输入商品详情...',
          onChange: (ed: any) => {
            emit('update:modelValue', ed.getHtml());
          },
        },
        html: props.modelValue || '<p></p>',
        mode: 'default',
      });
      editorRef.value = editor;

      WE.createToolbar({
        editor,
        selector: toolbarContainerRef.value,
        config: {},
        mode: 'default',
      });
    }

    watch(() => props.modelValue, (val) => {
      if (editorRef.value) {
        const current = editorRef.value.getHtml();
        if (val !== current) {
          editorRef.value.setHtml(val || '<p></p>');
        }
      }
    });

    onMounted(() => { initEditor(); });

    onBeforeUnmount(() => {
      if (editorRef.value) {
        editorRef.value.destroy();
        editorRef.value = null;
      }
    });

    return { editorContainerRef, toolbarContainerRef };
  },
});
</script>

<template>
  <div class="editor-container border border-gray-200 rounded-lg overflow-hidden" style="z-index: 10">
    <div ref="toolbarContainerRef" class="border-b border-gray-200"></div>
    <div ref="editorContainerRef" style="height: 480px;"></div>
  </div>
</template>
