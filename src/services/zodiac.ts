import type { ZodiacSign } from "@/types";

export const zodiacSigns: ZodiacSign[] = [
  { name: "摩羯座", slug: "capricorn", symbol: "♑" },
  { name: "水瓶座", slug: "aquarius", symbol: "♒" },
  { name: "双鱼座", slug: "pisces", symbol: "♓" },
  { name: "白羊座", slug: "aries", symbol: "♈" },
  { name: "金牛座", slug: "taurus", symbol: "♉" },
  { name: "双子座", slug: "gemini", symbol: "♊" },
  { name: "巨蟹座", slug: "cancer", symbol: "♋" },
  { name: "狮子座", slug: "leo", symbol: "♌" },
  { name: "处女座", slug: "virgo", symbol: "♍" },
  { name: "天秤座", slug: "libra", symbol: "♎" },
  { name: "天蝎座", slug: "scorpio", symbol: "♏" },
  { name: "射手座", slug: "sagittarius", symbol: "♐" },
];

export function zodiacFromBirthday(birthday: string): ZodiacSign {
  const match = /^\d{4}-(\d{2})-(\d{2})$/.exec(birthday);
  if (!match) return zodiacSigns[9];

  const marker = Number(match[1]) * 100 + Number(match[2]);
  const ranges: Array<[number, number, number]> = [
    [120, 218, 1],
    [219, 320, 2],
    [321, 419, 3],
    [420, 520, 4],
    [521, 621, 5],
    [622, 722, 6],
    [723, 822, 7],
    [823, 922, 8],
    [923, 1023, 9],
    [1024, 1122, 10],
    [1123, 1221, 11],
  ];

  if (marker >= 1222 || marker <= 119) return zodiacSigns[0];
  const range = ranges.find(([start, end]) => marker >= start && marker <= end);
  return zodiacSigns[range?.[2] ?? 9];
}

export function formatChineseDate(dateKey: string): string {
  const date = new Date(`${dateKey}T12:00:00`);
  return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
}
