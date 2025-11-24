import { useModelStore } from '@/store/model-store'

export function useModelState() {
  const {
    currentModelId,
    elementsById,
    selectedElementId,
    setCurrentModel,
    setElements,
    selectElement,
  } = useModelStore()

  const selectedElement = selectedElementId
    ? elementsById[selectedElementId]
    : undefined

  return {
    currentModelId,
    elementsById,
    selectedElementId,
    selectedElement,
    setCurrentModel,
    setElements,
    selectElement,
  }
}
