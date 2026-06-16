async function getCommonConfig() {
  return {
    build: {
      chunkSizeWarningLimit: 2000,
      reportCompressedSize: false,
      sourcemap: false,
    },
  };
}
export { getCommonConfig };
