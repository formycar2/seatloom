export const translations = {
  en: {
    sidebar: {
      inbox: '收件箱',
      timeline: '活动日志',
      workitems: '工作项',
      seats: '团队席位',
      sessions: '工作会话'
    },
    inbox: {
      title: '需要处理',
      clear: '状态已同步',
      clearSub: '所有任务已处理完毕',
      morningDigest: '今日协调摘要',
      sinceLast: '自上次会话以来：'
    },
    detail: {
      activeSessions: '当前正在进行的会话',
      assignedWI: '负责的工作项',
      source: '数据记录来源',
      mismatch: '同步警告：代码实现与需求不符',
      authority: '系统权威来源',
      purpose: '目的',
      expected: '预期结果',
      artifacts: '关联产物'
    },
    forms: {
      createWorkItem: '新建工作项',
      createHandoff: '发起交接',
      title: '标题',
      goal: '目标',
      priority: '优先级',
      assignee: '负责人',
      acceptanceCriteria: '验收标准',
      addCriteria: '添加标准',
      recipient: '接收席位',
      purpose: '目的',
      expected: '预期结果',
      attachArtifacts: '附加产物',
      receiptRequired: '需要确认回执'
    },
    terminal: {
      title: '终端实况',
      activeSession: '当前会话',
      noSession: '暂无活跃会话'
    },
    pipeline: {
      title: '执行进度',
      stages: '阶段',
      reconciling: '正在同步项目状态...',
      summary: '项目状态对齐：发现 {count} 处不一致',
      viewInInbox: '前往收件箱查看'
    },
    common: {
      cancel: '取消',
      create: '创建',
      initialize: '初始化',
      save: '保存',
      delete: '删除',
      close: '关闭',
      retry: '重试'
    }
  },
  zh: {
    sidebar: {
      inbox: '收件箱',
      timeline: '活动日志',
      workitems: '工作项',
      seats: '团队席位',
      sessions: '工作会话'
    },
    inbox: {
      title: '需要处理',
      clear: '状态已同步',
      clearSub: '所有任务已处理完毕',
      morningDigest: '今日协调摘要',
      sinceLast: '自上次会话以来：'
    },
    detail: {
      activeSessions: '当前正在进行的会话',
      assignedWI: '负责的工作项',
      source: '数据记录来源',
      mismatch: '同步警告：代码实现与需求不符',
      authority: '系统权威来源',
      purpose: '目的',
      expected: '预期结果',
      artifacts: '关联产物'
    },
    forms: {
      createWorkItem: '新建工作项',
      createHandoff: '发起交接',
      title: '标题',
      goal: '目标',
      priority: '优先级',
      assignee: '负责人',
      acceptanceCriteria: '验收标准',
      addCriteria: '添加标准',
      recipient: '接收席位',
      purpose: '目的',
      expected: '预期结果',
      attachArtifacts: '附加产物',
      receiptRequired: '需要确认回执'
    },
    terminal: {
      title: '终端实况',
      activeSession: '当前会话',
      noSession: '暂无活跃会话'
    },
    pipeline: {
      title: '执行进度',
      stages: '阶段',
      reconciling: '正在同步项目状态...',
      summary: '项目状态对齐：发现 {count} 处不一致',
      viewInInbox: '前往收件箱查看'
    },
    common: {
      cancel: '取消',
      create: '创建',
      initialize: '初始化',
      save: '保存',
      delete: '删除',
      close: '关闭',
      retry: '重试'
    }
  }
};

export type Locale = 'en' | 'zh';
