import { useEffect, useState } from "react";
import { api, events, unsubscribeAll } from "./api";

export type Lang = "vi" | "en";

/** Each language is named in itself, whatever the current interface language is. */
export const LANGUAGE_NAMES: Record<Lang, string> = { vi: "Tiếng Việt", en: "English" };

const vi = {
  "settings.windowTitle": "ToneSnap – Cài đặt",
  "settings.loading": "Đang tải…",
  "settings.leadStart": "Bôi đen đoạn văn bản (tiếng Việt, Anh hoặc Nhật) ở bất kỳ app nào, nhấn",
  "settings.leadEnd": "để nhận bản dịch hoặc bản viết lại theo văn phong bạn cần.",
  "settings.save": "Lưu Cài Đặt",
  "settings.needsAttention": "Cần xử lý",
  "settings.saveFailed": "Không lưu được cài đặt: {detail}",
  "settings.issue.invalid_fields":
    "Một số cài đặt đã lưu không hợp lệ nên đã về mặc định: {fields}. Bấm Lưu Cài Đặt để ghi lại.",
  "settings.issue.broken":
    "File cài đặt bị hỏng nên ToneSnap đang dùng cài đặt mặc định. File cũ được giữ tại {backup}.",
  "settings.issue.unreadable":
    "Chưa đọc được file cài đặt (có thể đang bị chương trình khác giữ) nên ToneSnap đang dùng cài đặt mặc định. Lưu Cài Đặt sẽ ghi đè file đó.",

  "section.interface": "Giao diện",
  "section.interface.desc": "Ngôn ngữ hiển thị và việc khởi động cùng Windows.",
  "uiLanguage.label": "Ngôn ngữ giao diện",
  "uiLanguage.auto": "Tự động (theo Windows)",
  "launchAtLogin.label": "Khởi động ToneSnap khi đăng nhập Windows",
  "launchAtLogin.help": "ToneSnap chờ sẵn ở khay hệ thống để dùng được ngay.",
  "launchAtLogin.failed": "Đã lưu cài đặt, nhưng không đổi được việc khởi động cùng Windows: {detail}",

  "section.ai": "Kết nối AI",
  "section.ai.desc": "Chọn nhà cung cấp AI để viết lại văn bản, rồi nhập API key của bạn.",
  "provider.gemini": "Google Gemini – có gói miễn phí",
  "provider.gemini.hint": "Khuyên dùng để bắt đầu. Lấy key miễn phí ở Google AI Studio, giới hạn số lượt gọi mỗi ngày.",
  "provider.groq": "Groq – có gói miễn phí, trả lời rất nhanh",
  "provider.groq.hint": "Lấy key miễn phí ở Groq Console, giới hạn số lượt gọi mỗi ngày.",
  "provider.openrouter": "OpenRouter – có các model miễn phí",
  "provider.openrouter.hint": "Một key dùng được nhiều model. Model miễn phí giới hạn khoảng 50 lượt mỗi ngày.",
  "provider.claude": "Claude API – trả phí, chất lượng cao nhất",
  "provider.claude.hint": "Tính phí theo lượng dùng, cần Anthropic API key.",
  "provider.freeNotice":
    "Gói miễn phí thường cho phép nhà cung cấp dùng nội dung bạn gửi để cải thiện sản phẩm. Dùng gói trả phí nếu văn bản có nội dung riêng tư.",
  "apiKey.label": "API key",
  "apiKey.get.claude": "Lấy key tại console.anthropic.com/settings/keys",
  "apiKey.get.gemini": "Lấy key miễn phí tại aistudio.google.com/apikey",
  "apiKey.get.groq": "Lấy key miễn phí tại console.groq.com/keys",
  "apiKey.get.openrouter": "Lấy key tại openrouter.ai/keys",
  "apiKey.placeholderSaved": "Đã lưu (nhập key mới để thay)",
  "apiKey.getKey": "Lấy Key",
  "apiKey.save": "Lưu Key",
  "apiKey.remove": "Xoá",
  "apiKey.help":
    "Key được lưu trong Windows Credential Manager, không ghi ra file. Nội dung chỉ gửi thẳng tới nhà cung cấp bạn chọn.",
  "apiKey.saved": "Đã lưu API key vào Windows Credential Manager.",
  "apiKey.removed": "Đã xoá API key.",
  "model.label": "Model",
  "model.custom": "Chọn từ gợi ý hoặc nhập tên model khác.",
  "model.suggestions": "Hiện các model gợi ý",
  "model.opus": "Chất lượng cao nhất",
  "model.sonnet": "Cân bằng tốc độ và chất lượng",
  "model.haiku": "Nhanh và rẻ nhất",
  "effort.label": "Mức suy nghĩ (effort)",
  "effort.low": "Thấp – nhanh nhất",
  "effort.medium": "Vừa",
  "effort.high": "Cao – chậm hơn",
  "effort.haikuNote": "Haiku 4.5 không hỗ trợ tuỳ chọn này.",

  "section.trigger": "Cách hiện popup",
  "section.trigger.desc": "Cách mở popup và phím tắt.",
  "trigger.hotkey": "Chỉ khi nhấn phím tắt",
  "trigger.hotkey.hint": "Bôi đen rồi nhấn phím tắt. Không làm phiền khi bạn chỉ đang đọc.",
  "trigger.icon": "Hiện icon nhỏ cạnh đoạn chọn",
  "trigger.icon.hint": "Bôi đen chữ là icon ToneSnap hiện cạnh con trỏ, bấm vào để mở popup. Phím tắt vẫn dùng được.",
  "trigger.auto": "Tự bật popup khi bôi đen",
  "trigger.auto.hint": "Popup tự mở sau một khoảng trễ, chỉ với chữ tiếng Anh.",
  "trigger.off": "Tắt",
  "trigger.off.hint": "Tạm dừng ToneSnap, kể cả phím tắt.",
  "trigger.soon": "Sắp có",
  "hotkey.label": "Phím tắt",
  "hotkey.help": "Bấm vào ô rồi nhấn tổ hợp phím mới (cần ít nhất một phím Ctrl/Alt/Shift/Win).",
  "hotkey.taken": "Phím tắt {hotkey} đang bị app khác dùng nên chưa hoạt động. Hãy chọn tổ hợp khác rồi Lưu.",
  "hotkey.invalid": "Phím tắt {hotkey} không hợp lệ: {detail}",
  "hotkey.conflict":
    "Phím này thường bị {apps} dùng: ToneSnap có thể không nhận được, hoặc sẽ chiếm phím tắt của app đó.",
  "autoDelay.label": "Độ trễ trước khi tự bật (ms)",

  "section.tones": "Ngôn ngữ và văn phong",
  "section.tones.desc": "Ngôn ngữ đích và chế độ mặc định của popup, cùng các văn phong hiển thị cho mỗi đoạn chọn.",
  "target.label": "Ngôn ngữ đích mặc định",
  "target.hint": "Mọi phiên bản đều viết bằng ngôn ngữ này, dù đoạn chọn là tiếng gì. Đổi nhanh được trong popup.",
  "target.vi": "Tiếng Việt",
  "target.en": "Tiếng Anh",
  "target.ja": "Tiếng Nhật",
  "mode.label": "Chế độ mặc định",
  "mode.translate": "Dịch",
  "mode.rewrite": "Viết lại",
  "mode.translate.hint": "Dịch sát nghĩa, giữ cấu trúc câu, chỉ chỉnh giọng theo văn phong.",
  "mode.rewrite.hint": "Diễn đạt lại tự do như người bản xứ viết: đổi cách nói, sắp xếp lại ý, sửa lỗi.",

  "section.context": "Ngữ cảnh",
  "section.context.desc": "Văn bản xung quanh đoạn chọn giúp AI hiểu tình huống.",
  "contextChars.label": "Số ký tự lấy mỗi bên đoạn chọn",
  "ocr.label": "Chụp màn hình + OCR khi app không cho đọc chữ",
  "ocr.help": "Ảnh chỉ xử lý trên máy bằng Windows OCR, không gửi đi. Chỉ phần chữ (đã che) được gửi.",

  "section.privacy": "Quyền riêng tư",
  "section.privacy.desc": "Kiểm soát dữ liệu rời khỏi máy và những app ToneSnap bỏ qua.",
  "redact.label": "Che dữ liệu nhạy cảm trước khi gửi",
  "redact.help": "Email, số điện thoại, CCCD, số thẻ, IP, URL, API key/token. Tự khôi phục khi hiển thị và Apply.",
  "preview.label": "Xem trước dữ liệu trước khi gửi",
  "preview.help": "Popup hiện đúng nội dung sẽ gửi đi, bạn bấm Gửi mới gọi AI.",
  "terms.label": "Từ cần che thêm (mỗi dòng một từ)",
  "terms.placeholder": "Tên khách hàng\nMã dự án",
  "blocked.label": "App bị chặn (tên tiến trình, mỗi dòng một app)",

  "section.updates": "Cập nhật",
  "section.updates.desc": "Phiên bản mới của ToneSnap trên GitHub và cách cài đặt chúng.",
  "updates.current": "Phiên bản hiện tại: {version}",
  "updateMode.auto": "Tự động cập nhật",
  "updateMode.auto.hint":
    "Tải ngầm, rồi cài và khởi động lại khi bạn không dùng ToneSnap (không mở popup hay cửa sổ Cài đặt).",
  "updateMode.manual": "Hỏi trước khi cập nhật",
  "updateMode.manual.hint":
    "Chỉ báo khi có phiên bản mới, ở khay hệ thống và tại đây. Bạn bấm Cập Nhật và Khởi Động Lại để cài.",
  "updates.status.idle": "Chưa kiểm tra phiên bản mới.",
  "updates.status.checking": "Đang kiểm tra phiên bản mới…",
  "updates.status.up_to_date": "Bạn đang dùng phiên bản mới nhất.",
  "updates.status.available": "Đã có phiên bản {version}.",
  "updates.status.downloading": "Đang tải phiên bản {version}…",
  "updates.status.ready.auto": "Đã tải phiên bản {version}. ToneSnap sẽ cài nó khi bạn không dùng ToneSnap.",
  "updates.status.ready.manual": "Đã tải phiên bản {version}, sẵn sàng để cài.",
  "updates.status.installing": "Đang cài phiên bản {version}. ToneSnap sẽ tự khởi động lại.",
  "updates.status.failed.check": "Không kiểm tra được phiên bản mới: {detail}",
  "updates.status.failed.download": "Không tải được phiên bản mới: {detail}",
  "updates.status.failed.install": "Không cài được phiên bản mới: {detail}",
  "updates.msiNote":
    "ToneSnap được cài bằng gói MSI nên mỗi lần cập nhật cần quyền quản trị. ToneSnap luôn hỏi trước khi cài, kể cả khi chọn tự động.",
  "updates.check": "Kiểm Tra Cập Nhật",
  "updates.install": "Cập Nhật và Khởi Động Lại",

  "tone.concise": "Ngắn gọn",
  "tone.concise.desc": "Bản ngắn nhất mà vẫn rõ ý",
  "tone.natural": "Tự nhiên",
  "tone.natural.desc": "Cách người bản xứ thường viết trong tình huống này",
  "tone.formal": "Trang trọng",
  "tone.formal.desc": "Văn phong công việc, dùng cho tài liệu, email gửi khách hàng hoặc sếp",

  "notice.no_selection": "Chưa có đoạn văn nào được chọn. Hãy bôi đen đoạn cần viết lại rồi nhấn phím tắt.",
  "notice.password_field": "ToneSnap không đọc nội dung trong ô mật khẩu.",
  "notice.blocked_app": "ToneSnap đang tắt trong {app} (danh sách app bị chặn trong Cài đặt).",
  "notice.apply_failed": "Không đưa được nội dung về app gốc. Đã copy sẵn, bạn nhấn Ctrl+V để dán.",

  "popup.empty": "Bôi đen một đoạn văn rồi nhấn phím tắt.",
  "popup.close": "Đóng",
  "popup.closeTitle": "Đóng (Esc)",
  "popup.settingsTitle": "Cài đặt",
  "popup.source.uia": "Đọc trực tiếp",
  "popup.source.clipboard": "Qua clipboard",
  "popup.context.uia": "Ngữ cảnh: văn bản xung quanh",
  "popup.context.ocr": "Ngữ cảnh: OCR màn hình",
  "popup.context.none": "Không có ngữ cảnh",
  "popup.redacted": "Đã che {n} mục nhạy cảm",
  "popup.readOnly": "Chỉ đọc – dùng Copy",
  "popup.options.language": "Ngôn ngữ đích",
  "popup.options.mode": "Chế độ",
  "popup.apply": "Apply",
  "popup.copy": "Copy",
  "popup.writing": "Đang viết…",
  "popup.regenerate": "Tạo lại",
  "popup.retry": "Thử Lại (R)",
  "popup.openSettings": "Mở Cài Đặt",
  "preview.title": "Dữ liệu sẽ gửi đi",
  "preview.app": "App",
  "preview.before": "Phía trước",
  "preview.selection": "Đoạn chọn",
  "preview.target": "Ngôn ngữ đích",
  "preview.mode": "Chế độ",
  "preview.after": "Phía sau",
  "preview.screen": "Chữ trên màn hình (OCR)",
  "preview.send": "Gửi (Enter)",
  "preview.cancel": "Huỷ",

  "error.no_api_key": "Chưa có API key",
  "error.no_api_key.detail": "Mở Cài đặt để nhập API key.",
  "error.authentication_error": "API key không hợp lệ",
  "error.authentication_error.detail": "Kiểm tra lại key trong Cài đặt.",
  "error.permission_error": "Key không có quyền dùng model này",
  "error.not_found_error": "Không tìm thấy model",
  "error.rate_limit_error": "Đang bị giới hạn tần suất gọi API",
  "error.rate_limit_error.detail": "Thử lại sau vài giây (phím R).",
  "error.overloaded_error": "Máy chủ AI đang quá tải",
  "error.overloaded_error.detail": "Thử lại sau ít phút (phím R).",
  "error.refusal": "AI từ chối xử lý đoạn này",
  "error.max_tokens": "Kết quả quá dài nên bị cắt",
  "error.max_tokens.detail": "Hãy chọn đoạn ngắn hơn rồi thử lại.",
  "error.network": "Lỗi kết nối",
  "error.generic": "Lỗi {status}",
};

export type Key = keyof typeof vi;

const en: Record<Key, string> = {
  "settings.windowTitle": "ToneSnap – Settings",
  "settings.loading": "Loading…",
  "settings.leadStart": "Select Vietnamese, English or Japanese text in any app and press",
  "settings.leadEnd": "to get a translation or a rewrite in the tone you need.",
  "settings.save": "Save Settings",
  "settings.needsAttention": "Needs attention",
  "settings.saveFailed": "Could not save settings: {detail}",
  "settings.issue.invalid_fields":
    "Some saved settings were not valid and went back to their defaults: {fields}. Click Save Settings to store the fix.",
  "settings.issue.broken":
    "The settings file was damaged, so ToneSnap is using the defaults. The old file was kept at {backup}.",
  "settings.issue.unreadable":
    "The settings file could not be read (another program may be holding it), so ToneSnap is using the defaults. Saving Settings will replace that file.",

  "section.interface": "Interface",
  "section.interface.desc": "Display language and starting with Windows.",
  "uiLanguage.label": "Interface language",
  "uiLanguage.auto": "Automatic (follow Windows)",
  "launchAtLogin.label": "Start ToneSnap when I sign in to Windows",
  "launchAtLogin.help": "ToneSnap waits in the tray so it is ready at once.",
  "launchAtLogin.failed": "Settings were saved, but starting with Windows could not be changed: {detail}",

  "section.ai": "AI connection",
  "section.ai.desc": "Pick the AI provider that rewrites your text, then enter your API key.",
  "provider.gemini": "Google Gemini – free tier",
  "provider.gemini.hint": "Recommended to start. Get a free key from Google AI Studio; daily request limits apply.",
  "provider.groq": "Groq – free tier, very fast",
  "provider.groq.hint": "Get a free key from the Groq Console; daily request limits apply.",
  "provider.openrouter": "OpenRouter – free models",
  "provider.openrouter.hint": "One key for many models. Free models are limited to about 50 requests a day.",
  "provider.claude": "Claude API – paid, highest quality",
  "provider.claude.hint": "Billed per use; needs an Anthropic API key.",
  "provider.freeNotice":
    "Free tiers usually let the provider use what you send to improve its products. Use a paid plan if your text is private.",
  "apiKey.label": "API key",
  "apiKey.get.claude": "Get a key at console.anthropic.com/settings/keys",
  "apiKey.get.gemini": "Get a free key at aistudio.google.com/apikey",
  "apiKey.get.groq": "Get a free key at console.groq.com/keys",
  "apiKey.get.openrouter": "Get a key at openrouter.ai/keys",
  "apiKey.placeholderSaved": "Saved (enter a new key to replace it)",
  "apiKey.getKey": "Get Key",
  "apiKey.save": "Save Key",
  "apiKey.remove": "Remove",
  "apiKey.help":
    "The key is stored in Windows Credential Manager, never in a file. Text is sent only to the provider you choose.",
  "apiKey.saved": "API key saved to Windows Credential Manager.",
  "apiKey.removed": "API key removed.",
  "model.label": "Model",
  "model.custom": "Pick a suggestion or type another model name.",
  "model.suggestions": "Show suggested models",
  "model.opus": "Highest quality",
  "model.sonnet": "Balanced speed and quality",
  "model.haiku": "Fastest and cheapest",
  "effort.label": "Reasoning effort",
  "effort.low": "Low – fastest",
  "effort.medium": "Medium",
  "effort.high": "High – slower",
  "effort.haikuNote": "Haiku 4.5 does not support this option.",

  "section.trigger": "Popup trigger",
  "section.trigger.desc": "How the popup opens, and its hotkey.",
  "trigger.hotkey": "Only when I press the hotkey",
  "trigger.hotkey.hint": "Select text, then press the hotkey. Stays out of the way while you read.",
  "trigger.icon": "Show a small icon next to the selection",
  "trigger.icon.hint":
    "Select text and the ToneSnap icon appears by the cursor; click it to open the popup. The hotkey still works.",
  "trigger.auto": "Open the popup automatically on selection",
  "trigger.auto.hint": "Opens after a short delay, only for English text.",
  "trigger.off": "Off",
  "trigger.off.hint": "Pause ToneSnap, including the hotkey.",
  "trigger.soon": "Coming soon",
  "hotkey.label": "Hotkey",
  "hotkey.help": "Click the box and press a new key combination (with at least one of Ctrl/Alt/Shift/Win).",
  "hotkey.taken":
    "{hotkey} is already used by another app, so it is not active. Choose a different combination and save.",
  "hotkey.invalid": "{hotkey} is not a valid hotkey: {detail}",
  "hotkey.conflict": "Commonly used by {apps}: ToneSnap may not receive it, or will take that app's shortcut away.",
  "autoDelay.label": "Delay before opening (ms)",

  "section.tones": "Language and tones",
  "section.tones.desc": "The popup's default target language and mode, and the tones shown for each selection.",
  "target.label": "Default target language",
  "target.hint":
    "Every version is written in this language, whatever language the selection is in. Change it quickly in the popup.",
  "target.vi": "Vietnamese",
  "target.en": "English",
  "target.ja": "Japanese",
  "mode.label": "Default mode",
  "mode.translate": "Translate",
  "mode.rewrite": "Rewrite",
  "mode.translate.hint": "Stays close to the meaning and sentence structure; only the tone is adjusted.",
  "mode.rewrite.hint": "Rephrases freely the way a native writer would: new wording, reordered ideas, fixed mistakes.",

  "section.context": "Context",
  "section.context.desc": "Text around the selection helps the AI understand the situation.",
  "contextChars.label": "Characters of context on each side",
  "ocr.label": "Screenshot + OCR when an app hides its text",
  "ocr.help":
    "The image is processed on this PC with Windows OCR and never sent. Only the recognized text, masked, is sent.",

  "section.privacy": "Privacy",
  "section.privacy.desc": "Control what leaves this PC and which apps ToneSnap ignores.",
  "redact.label": "Mask sensitive data before sending",
  "redact.help":
    "Emails, phone numbers, ID and card numbers, IPs, URLs, API keys and tokens. Restored automatically when shown and applied.",
  "preview.label": "Preview data before sending",
  "preview.help": "The popup shows exactly what will be sent; nothing goes out until you press Send.",
  "terms.label": "Extra terms to mask (one per line)",
  "terms.placeholder": "Client name\nProject code",
  "blocked.label": "Blocked apps (process name, one per line)",

  "section.updates": "Updates",
  "section.updates.desc": "New versions of ToneSnap on GitHub, and how they are installed.",
  "updates.current": "Current version: {version}",
  "updateMode.auto": "Update automatically",
  "updateMode.auto.hint":
    "Downloads in the background, then installs and restarts while you are not using ToneSnap (no popup or Settings window open).",
  "updateMode.manual": "Ask before updating",
  "updateMode.manual.hint":
    "Only tells you when a new version is out, in the tray and here. Click Update & Restart to install it.",
  "updates.status.idle": "Not checked for a new version yet.",
  "updates.status.checking": "Checking for a new version…",
  "updates.status.up_to_date": "You have the latest version.",
  "updates.status.available": "Version {version} is available.",
  "updates.status.downloading": "Downloading version {version}…",
  "updates.status.ready.auto": "Version {version} is downloaded. ToneSnap installs it while you are not using it.",
  "updates.status.ready.manual": "Version {version} is downloaded and ready to install.",
  "updates.status.installing": "Installing version {version}. ToneSnap will restart by itself.",
  "updates.status.failed.check": "Could not check for a new version: {detail}",
  "updates.status.failed.download": "Could not download the new version: {detail}",
  "updates.status.failed.install": "Could not install the new version: {detail}",
  "updates.msiNote":
    "ToneSnap was installed with the MSI package, so each update needs administrator rights. It always asks before installing, even in automatic mode.",
  "updates.check": "Check for Updates",
  "updates.install": "Update & Restart",

  "tone.concise": "Concise",
  "tone.concise.desc": "The shortest version that is still clear",
  "tone.natural": "Natural",
  "tone.natural.desc": "How a native speaker would usually put it here",
  "tone.formal": "Formal",
  "tone.formal.desc": "Professional register for documents and emails to clients or managers",

  "notice.no_selection": "Nothing is selected. Select the text you want to rewrite, then press the hotkey.",
  "notice.password_field": "ToneSnap never reads password fields.",
  "notice.blocked_app": "ToneSnap is turned off in {app} (blocked apps in Settings).",
  "notice.apply_failed": "Could not put the text back into the app. It is on your clipboard; press Ctrl+V to paste.",

  "popup.empty": "Select some text, then press the hotkey.",
  "popup.close": "Close",
  "popup.closeTitle": "Close (Esc)",
  "popup.settingsTitle": "Settings",
  "popup.source.uia": "Read directly",
  "popup.source.clipboard": "Via clipboard",
  "popup.context.uia": "Context: surrounding text",
  "popup.context.ocr": "Context: screen OCR",
  "popup.context.none": "No context",
  "popup.redacted": "{n} sensitive items masked",
  "popup.readOnly": "Read-only – use Copy",
  "popup.options.language": "Target language",
  "popup.options.mode": "Mode",
  "popup.apply": "Apply",
  "popup.copy": "Copy",
  "popup.writing": "Writing…",
  "popup.regenerate": "Regenerate",
  "popup.retry": "Retry (R)",
  "popup.openSettings": "Open Settings",
  "preview.title": "Data to be sent",
  "preview.app": "App",
  "preview.before": "Before",
  "preview.selection": "Selection",
  "preview.target": "Target language",
  "preview.mode": "Mode",
  "preview.after": "After",
  "preview.screen": "Screen text (OCR)",
  "preview.send": "Send (Enter)",
  "preview.cancel": "Cancel",

  "error.no_api_key": "No API key yet",
  "error.no_api_key.detail": "Open Settings to enter your API key.",
  "error.authentication_error": "Invalid API key",
  "error.authentication_error.detail": "Check the key in Settings.",
  "error.permission_error": "This key cannot use this model",
  "error.not_found_error": "Model not found",
  "error.rate_limit_error": "Rate limited",
  "error.rate_limit_error.detail": "Try again in a few seconds (R).",
  "error.overloaded_error": "The AI service is overloaded",
  "error.overloaded_error.detail": "Try again in a few minutes (R).",
  "error.refusal": "The AI declined to process this text",
  "error.max_tokens": "The result was too long and got cut off",
  "error.max_tokens.detail": "Select a shorter passage and try again.",
  "error.network": "Connection error",
  "error.generic": "Error {status}",
};

const DICTIONARIES: Record<Lang, Record<Key, string>> = { vi, en };

export const KEYS = Object.keys(vi) as Key[];

export function resolveLang(setting: string | undefined): Lang {
  if (setting === "vi" || setting === "en") return setting;
  return navigator.language.toLowerCase().startsWith("vi") ? "vi" : "en";
}

export type Translate = (key: Key, params?: Record<string, string | number>) => string;

export function translator(lang: Lang): Translate {
  return (key, params) =>
    Object.entries(params ?? {}).reduce(
      (text, [name, value]) => text.split(`{${name}}`).join(String(value)),
      DICTIONARIES[lang][key],
    );
}

export function hasKey(key: string): key is Key {
  return key in vi;
}

/** Interface language from saved settings, updated when Settings are saved. */
export function useSavedLang(): Lang {
  const [lang, setLang] = useState<Lang>(() => resolveLang(undefined));
  useEffect(() => {
    void api.getSettings().then((s) => setLang(resolveLang(s.uiLanguage)));
    return unsubscribeAll([events.settingsChanged((s) => setLang(resolveLang(s.uiLanguage)))]);
  }, []);
  return lang;
}
