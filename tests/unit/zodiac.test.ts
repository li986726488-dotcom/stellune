import { describe, expect, it } from "vitest";
import {
  formatChineseDate,
  zodiacFromBirthday,
} from "../../src/services/zodiac";

describe("zodiacFromBirthday", () => {
  it.each([
    ["1998-10-08", "libra", "天秤座"],
    ["2000-01-19", "capricorn", "摩羯座"],
    ["2000-01-20", "aquarius", "水瓶座"],
    ["2000-03-21", "aries", "白羊座"],
  ])("maps %s to %s", (birthday, slug, name) => {
    expect(zodiacFromBirthday(birthday)).toMatchObject({ slug, name });
  });
});

describe("formatChineseDate", () => {
  it("formats the fixture date without timezone drift", () => {
    expect(formatChineseDate("2026-07-23")).toBe("2026年7月23日");
  });
});
