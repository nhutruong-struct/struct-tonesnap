export type Lang = "en" | "ja";
export type Mode = "translate" | "rewrite";

export const SOURCE = "Anh xem giúp em PR này với, gấp lắm ạ";

export const TONES = ["Concise", "Natural", "Formal"] as const;

export const VARIANTS: Record<Lang, Record<Mode, [string, string, string]>> = {
  en: {
    translate: [
      "Could you review my PR? It's urgent.",
      "Could you take a look at my PR when you get a chance? It's pretty urgent.",
      "Would you mind reviewing my pull request at your earliest convenience? It is rather urgent.",
    ],
    rewrite: [
      "Please review my PR, it's urgent.",
      "Hey, could you review my PR today? It's time-sensitive.",
      "I would appreciate it if you could review my pull request today, as it is time-sensitive.",
    ],
  },
  ja: {
    translate: [
      "PRのレビューをお願いします。急ぎです。",
      "お手すきの際にPRを見ていただけますか？少し急いでいます。",
      "恐れ入りますが、PRのレビューを早急にお願いできますでしょうか。",
    ],
    rewrite: [
      "PRを確認してください。急ぎです。",
      "もしよければ、今日中にPRを見てもらえますか？",
      "お忙しいところ恐縮ですが、本日中にPRをご確認いただけますと幸いです。",
    ],
  },
};
