// A write-only form must say at save time that this build cannot store it.
//
// The defect (2026-10-16): four forms accept input and imply success, main.rs registers
// no create command for workitem / handoff / seat / delegation, and the forms make zero
// api.* calls — so the data is gone on reload while the button said 创建并写入. The
// dispatch's rule is one of two per form: wire a real command, or say so. There is no
// command, so this is the second option, and the missing command is named rather than
// invented.

import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import CannotStoreNotice, { MISSING_COMMANDS } from './CannotStoreNotice';

describe('CannotStoreNotice', () => {
  it('says the build cannot store it, and that closing discards the input', () => {
    render(<CannotStoreNotice kind="workitem" />);
    expect(screen.getByText(/这个构建存不下它/)).toBeTruthy();
    expect(screen.getByText(/关掉这个窗口,你刚填的就没有了/)).toBeTruthy();
  });

  it('names the exact missing command instead of inventing one', () => {
    render(<CannotStoreNotice kind="handoff" />);
    expect(screen.getByText(MISSING_COMMANDS.handoff)).toBeTruthy();
    expect(MISSING_COMMANDS).toEqual({
      workitem: 'cmd_create_workitem',
      handoff: 'cmd_create_handoff',
      seat: 'cmd_create_seat',
      delegation: 'cmd_create_delegation',
    });
  });

  it('explains that the missing thing is a backend command, not a button', () => {
    render(<CannotStoreNotice kind="seat" />);
    expect(screen.getByText(/缺的是后端一条命令,不是界面一个按钮/)).toBeTruthy();
  });

  it('never claims the data was saved', () => {
    render(<CannotStoreNotice kind="delegation" />);
    const text = document.body.textContent ?? '';
    expect(text).not.toContain('已保存');
    expect(text).not.toContain('创建成功');
    expect(text).not.toContain('已写入');
  });
});
