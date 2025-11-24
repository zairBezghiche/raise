import { useEffect } from 'react'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

export function useTauriEvent<T = unknown>(
  eventName: string,
  handler: (payload: T) => void,
) {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined

    listen<T>(eventName, (event) => {
      handler(event.payload)
    })
      .then((fn) => {
        unlisten = fn
      })
      .catch((err) => {
        console.error('[useTauriEvent] listen error', err)
      })

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [eventName, handler])
}
