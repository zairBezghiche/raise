import type { ReactNode } from 'react'

export interface TreeNode {
  id: string
  label: ReactNode
  children?: TreeNode[]
}

interface TreeViewProps {
  nodes: TreeNode[]
  level?: number
}

export function TreeView({ nodes, level = 0 }: TreeViewProps) {
  return (
    <ul
      style={{
        listStyle: 'none',
        paddingLeft: level === 0 ? 0 : 12,
        margin: 0,
        fontSize: 13,
      }}
    >
      {nodes.map((n) => (
        <li key={n.id} style={{ marginBottom: 4 }}>
          <div>{n.label}</div>
          {n.children && n.children.length > 0 && (
            <TreeView nodes={n.children} level={level + 1} />
          )}
        </li>
      ))}
    </ul>
  )
}
