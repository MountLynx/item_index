import { readFileSync, writeFileSync } from 'fs'

const css = readFileSync('node_modules/@tabler/icons-webfont/dist/tabler-icons.css', 'utf8')
const names = [...css.matchAll(/\.ti-([^:]+):before/g)].map(m => m[1])
const unique = [...new Set(names)].sort()

// Group by first dash-separated segment
const groups = {}
for (const n of unique) {
  const dash = n.indexOf('-')
  const prefix = dash > 0 ? n.slice(0, dash) : '_base'
  if (!groups[prefix]) groups[prefix] = []
  groups[prefix].push(n)
}

// Chinese labels
const labels = {
  _base: '基础', arrow: '箭头', arrows: '方向', brand: '品牌',
  device: '设备', file: '文件', folder: '文件夹', message: '消息',
  chart: '图表', circle: '圆形', square: '方形', triangle: '三角',
  currency: '货币', user: '用户', map: '地图', lock: '安全',
  photo: '图片', video: '视频', music: '音乐', camera: '相机',
  cloud: '云端', calendar: '日历', clock: '时间', heart: '爱心',
  star: '星级', flag: '旗帜', bell: '通知', mail: '邮件',
  phone: '电话', world: '地球', home: '家居', building: '建筑',
  shield: '盾牌', eye: '可见', number: '数字', letter: '字母',
  mood: '表情', math: '数学', filter: '过滤', layout: '布局',
  shopping: '购物', box: '盒子', hexagon: '六边',
  navigation: '导航', wash: '洗涤', align: '对齐',
  code: '代码', database: '数据库', server: '服务器', settings: '设置',
  search: '搜索', game: '游戏', tool: '工具', tools: '工具',
  color: '颜色', edit: '编辑', view: '视图', shape: '形状',
  medical: '医疗', sport: '运动', travel: '旅行',
  animal: '动物', food: '食物', weather: '天气',
  network: '网络', battery: '电池', key: '钥匙', book: '书籍',
  school: '学校', document: '文档', time: '时间', car: '汽车',
  badge: '徽章', award: '奖项', ball: '球类', bike: '自行车',
  bottle: '瓶子', brain: '大脑', bridge: '桥梁', brush: '画笔',
  bug: '缺陷', bulb: '灯泡', bus: '巴士', calculator: '计算器',
  card: '卡片', cash: '现金', coffee: '咖啡', compass: '指南针',
  copy: '复制', crown: '王冠', cut: '剪切', diamond: '钻石',
  door: '门', download: '下载', drop: '水滴', ear: '耳朵',
  face: '面部', fire: '火焰', fish: '鱼', flower: '花',
  frame: '框架', gift: '礼物', glass: '玻璃', globe: '地球仪',
  guitar: '吉他', hammer: '锤子', headphone: '耳机', help: '帮助',
  history: '历史', ice: '冰', info: '信息', label: '标签',
  lamp: '灯', language: '语言', layer: '图层', leaf: '叶子',
  life: '生命', light: '光', link: '链接', list: '列表',
  loader: '加载', location: '位置', login: '登录', logout: '登出',
  magnet: '磁铁', mark: '标记', mask: '面具', medal: '奖牌',
  menu: '菜单', mic: '麦克', minus: '减号', mirror: '镜子',
  money: '金钱', moon: '月亮', mouse: '鼠标', move: '移动',
  movie: '电影', news: '新闻', note: '笔记', package: '包裹',
  page: '页面', paint: '油漆', palette: '调色板', paper: '纸张',
  paste: '粘贴', pause: '暂停', peace: '和平', pencil: '铅笔',
  phone: '电话', photo: '图片', pie: '饼图', pin: '图钉',
  pizza: '披萨', plane: '飞机', planet: '行星', play: '播放',
  plug: '插头', plus: '加号', power: '电源', print: '打印',
  puzzle: '拼图', radio: '收音机', rain: '雨', receipt: '收据',
  refresh: '刷新', rocket: '火箭', route: '路线', ruler: '尺子',
  save: '保存', scale: '比例', scan: '扫描', scissors: '剪刀',
  screen: '屏幕', send: '发送', share: '分享', ship: '船',
  shirt: '衬衫', shoe: '鞋', skull: '骷髅', sleep: '睡眠',
  smart: '智能', snow: '雪', sort: '排序', speaker: '扬声器',
  speed: '速度', stack: '堆叠', stop: '停止', sun: '太阳',
  swim: '游泳', switch: '开关', table: '表格', tag: '标签',
  target: '目标', test: '测试', text: '文本', thumb: '拇指',
  ticket: '票券', toggle: '切换', train: '火车', trash: '删除',
  tree: '树', truck: '卡车', umbrella: '雨伞', undo: '撤销',
  upload: '上传', vaccine: '疫苗', volume: '音量', wallet: '钱包',
  watch: '手表', water: '水', wave: '波浪', weight: '重量',
  wheel: '车轮', wifi: 'WiFi', wind: '风', window: '窗口',
  wine: '红酒', wrench: '扳手', write: '书写', zoom: '缩放',
}

// Build groups: known labels stay, small groups (< 8 icons) merged into "其他"
const out = []
const other = []
for (const [prefix, icons] of Object.entries(groups)) {
  if (labels[prefix] || icons.length >= 15) {
    out.push({ prefix, label: labels[prefix] || prefix, icons })
  } else {
    other.push(...icons)
  }
}
if (other.length) {
  other.sort()
  out.push({ prefix: '_other', label: '其他', icons: other })
}

out.sort((a, b) => {
  const aKnown = labels[a.prefix] ? 0 : 1
  const bKnown = labels[b.prefix] ? 0 : 1
  if (aKnown !== bKnown) return aKnown - bKnown
  if (a.prefix === '_other') return 1
  if (b.prefix === '_other') return -1
  return a.prefix.localeCompare(b.prefix)
})

let ts = '// Auto-generated from @tabler/icons-webfont\n'
ts += `// ${unique.length} icons in ${out.length} groups\n\n`
ts += 'export interface IconGroup { prefix: string; label: string; icons: string[] }\n\n'
ts += 'export const ICON_GROUPS: IconGroup[] = [\n'
for (const g of out) {
  ts += `  { prefix: '${g.prefix}', label: '${g.label}', icons: [`
  for (let i = 0; i < g.icons.length; i += 12) {
    ts += '\n    ' + g.icons.slice(i, i + 12).map(s => `'${s}'`).join(', ') + ','
  }
  ts += '\n  ] },\n'
}
ts += ']\n'

writeFileSync('src/assets/icon-names.ts', ts)
console.log(`Generated ${unique.length} icons in ${out.length} groups`)
