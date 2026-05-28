import { describe, it, expect } from 'vitest'
import { bytesToSize } from '@shared/utils'

// Minimal smoke test: verifies the shared util + alias resolution work.
describe('bytesToSize', () => {
  it('formats zero', () => {
    expect(bytesToSize(0)).toBe('0 KB')
  })

  it('formats bytes', () => {
    expect(bytesToSize(512)).toBe('512 B')
  })

  it('formats kilobytes with precision', () => {
    expect(bytesToSize(1536, 1)).toBe('1.5 KB')
  })

  it('formats megabytes', () => {
    expect(bytesToSize(1048576)).toBe('1.0 MB')
  })
})
