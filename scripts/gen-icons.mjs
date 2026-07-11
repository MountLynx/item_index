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

// Chinese labels for common prefixes
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
  navigation: '导航', wash: '洗涤', align: '对齐', settings: '设置',
  search: '搜索', game: '游戏', tool: '工具', tools: '工具',
  color: '颜色', edit: '编辑', view: '视图', shape: '形状',
  medical: '医疗', sport: '运动', travel: '旅行', weather: '天气',
  network: '网络', battery: '电池', keyboard: '键盘', book: '书籍',
  school: '学校', car: '汽车', badge: '徽章', award: '奖项',
  ball: '球类', bike: '自行车', brain: '大脑', bulb: '灯泡',
  bus: '巴士', card: '卡片', cash: '现金', coffee: '咖啡',
  compass: '指南针', crown: '王冠', diamond: '钻石', download: '下载',
  upload: '上传', fire: '火焰', flower: '花', gift: '礼物',
  globe: '地球仪', guitar: '吉他', headphone: '耳机', key: '钥匙',
  layer: '图层', light: '光', link: '链接', list: '列表',
  money: '金钱', moon: '月亮', movie: '电影', music: '音乐',
  note: '笔记', paint: '油漆', palette: '调色板', pencil: '铅笔',
  play: '播放', rocket: '火箭', save: '保存', send: '发送',
  share: '分享', ship: '船', sun: '太阳', table: '表格',
  tag: '标签', trash: '删除', tree: '树', truck: '卡车',
  volume: '音量', wallet: '钱包', watch: '手表', wind: '风',
  wine: '红酒', zoom: '缩放',
}

// Known labels stay, small groups (< 15 icons) → "其他"
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
ts += `// ${unique.length} icons in ${out.length} groups\n`
ts += '// Regenerate: node scripts/gen-icons.mjs\n\n'
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
