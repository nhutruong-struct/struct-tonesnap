# Local model benchmark – 2026-10-07

GPU idle: 16.6 W, 44 °C. Tones: minimal, concise, polite. Explanations: Vietnamese.

| Model | Valid | Explanations in Vietnamese | First token (median) | Total (median) | Output tok/s | Avg / peak GPU W | Peak °C | Load |
|---|---|---|---|---|---|---|---|---|
| qwen3.5:4b | 11/11 | 53/53 | 0.19s | 3.02s | 111 | 107.3 / 110.9 | 63 | 2.9s |

## slack-urgent-pr (Slack)

> pls check my PR, it urgent

**qwen3.5:4b** – 2.8s

- *minimal*: Please check my PR; it's urgent.  
  Đã thêm dấu chấm than và từ 'it's' để sửa lỗi ngữ pháp.
- *concise*: PR is urgent, please check.  
  Ngắn gọn hơn bằng cách đảo cấu trúc câu.
- *polite*: Could you please review my PR? It's quite urgent.  
  Sử dụng từ 'Could you' và 'quite' để làm mềm lời yêu cầu.
- ✎ pls → Please: Viết đầy đủ 'Please' thay vì viết tắt 'pls' trong văn bản công việc.
- ✎ it urgent → it's urgent: Cần thêm dấu ngoặc đơn 'it's' để nối chủ ngữ và tính từ.

## outlook-delay (Outlook)

> I want to inform that the meeting tomorrow will be delay to 3pm because customer is busy.

**qwen3.5:4b** – 3.0s

- *minimal*: I want to inform you that the meeting tomorrow will be delayed to 3pm because the customer is busy.  
  Chỉ sửa lỗi ngữ pháp và từ vựng, giữ nguyên cấu trúc câu.
- *concise*: Please note that the meeting tomorrow has been rescheduled to 3pm due to the customer's schedule.  
  Ngắn gọn hơn, dùng từ 'rescheduled' thay vì giải thích dài dòng.
- *polite*: I am writing to let you know that the meeting scheduled for tomorrow will need to be moved to 3pm as the customer is currently unavailable.  
  Giữ sự lịch sự và tôn trọng, phù hợp cho email chuyên nghiệp.
- ✎ I want to inform that → I want to inform you that: Thiếu tân ngữ 'you' sau từ 'inform'.

## slack-bug (Slack)

> We has finished the task but there is some bug need to fix before release.

**qwen3.5:4b** – 3.1s

- *minimal*: We have finished the task, but there is some bug to fix before release.  
  Đã sửa lỗi ngữ pháp và từ vựng; giữ nguyên cấu trúc câu gốc.
- *concise*: Task done, but a bug needs fixing before release.  
  Ngắn gọn hơn bằng cách loại bỏ các từ thừa.
- *polite*: We have completed the task; however, there is a bug that needs to be fixed before release.  
  Đã dùng từ ngữ lịch sự và cấu trúc câu mượt mà hơn để diễn đạt yêu cầu.
- ✎ We has finished → We have finished: Lỗi ngữ pháp: 'has' không đúng với chủ ngữ số nhiều 'We'; cần dùng 'have'.
- ✎ there is some bug need to fix → there is a bug that needs to be fixed: Lỗi cú pháp: Thiếu từ nối và động từ bổ nghĩa cho 'bug'; cần thêm 'that' và chia đúng thì.

## teams-already-fine (Microsoft Teams)

> Could you review this document and share your feedback before Friday?

**qwen3.5:4b** – 2.8s

- *minimal*: Could you review this document and share your feedback before Friday?  
  
- *concise*: Please review the document and share feedback by Friday.  
  Giảm bớt các từ thừa, giữ nguyên ý nghĩa.
- *polite*: Would you mind reviewing this document and sharing your feedback before Friday?  
  Thêm lời hỏi ý kiến để mềm mỏng hơn.
- ✎ Could you review this document → Could you please review this document: Thêm 'please' để lịch sự hơn trong văn bản công việc.
- ✎ share your feedback → share your feedback: Đã đúng, không cần sửa.
- ✎ before Friday → by Friday: 'By Friday' thường dùng hơn cho thời hạn (deadline) trong tiếng Anh.

## vi-weekly-report (Slack)

> Anh ơi em gửi báo cáo tuần rồi nhé, anh xem giúp em với

**qwen3.5:4b** – 2.5s

- *minimal*: Hey, I've sent the weekly report, could you take a look?  
  
- *concise*: Sent the weekly report; please review.  
  
- *polite*: Hi, I've just sent the weekly report. Would you mind taking a look when you have a moment?  
  
- ✎ Anh ơi em gửi báo cáo tuần rồi nhé → Hey, I've sent the weekly report: Dịch từ tiếng Việt sang tiếng Anh tự nhiên trong ngữ cảnh chat.
- ✎ anh xem giúp em với → could you take a look?: Biểu thị sự cầu khiến lịch sự thay vì 'giúp'.

## mixed-deploy (Slack)

> em đã fix bug login rồi, nhưng chưa deploy lên staging được vì server down

**qwen3.5:4b** – 3.5s

- *minimal*: I've fixed the login bug, but haven't deployed to staging yet because the server is down.  
  Dịch và sửa lỗi ngữ pháp cơ bản, giữ nguyên cấu trúc câu gốc.
- *concise*: Login bug fixed; can't deploy to staging due to server downtime.  
  Ngắn gọn hơn bằng cách lược bỏ từ thừa và dùng dấu chấm phẩy.
- *polite*: I've managed to fix the login bug, but we're still unable to deploy to staging as the server is currently down.  
  Thêm các từ làm mềm giọng điệu như 'managed to' và 'currently' để lịch sự hơn.
- ✎ em đã fix bug login rồi → I've fixed the login bug: Thay 'em' bằng 'I' và thêm trợ động từ 'have' để hoàn thành hành động.
- ✎ chưa deploy lên staging được → haven't deployed to staging yet: Sửa cấu trúc câu bị động/thành ngữ thành tiếng Anh tự nhiên hơn.
- ✎ vì server down → because the server is down: Thêm mạo từ 'the' và trợ động từ 'is' để hoàn chỉnh câu.

## email-late-reply (Outlook)

> Sorry for late reply, I was busy with other project so I didn't see your email.

**qwen3.5:4b** – 2.8s

- *minimal*: Sorry for the late reply; I was busy with another project so I didn't see your email.  
  Đã sửa lỗi ngữ pháp và thêm từ nối phù hợp.
- *concise*: Apologies for the delay; I missed your email due to a busy schedule.  
  Ngắn gọn hơn, loại bỏ các từ thừa.
- *polite*: I apologize for my delayed response. I was occupied with another project and unfortunately didn't see your email.  
  Nói lịch sự hơn, thể hiện sự tiếc nuối và tôn trọng người nhận.
- ✎ Sorry for late reply → Sorry for the late reply: Thêm từ 'the' để cấu trúc đúng ngữ pháp.
- ✎ busy with other project → busy with another project: Sửa số lượng từ 'project' thành dạng số ít phù hợp với 'another'.

## code-review (Google Chrome)

> This function return null when input is empty, I think we should handle this case.

**qwen3.5:4b** – 3.0s

- *minimal*: This function returns null when input is empty; I think we should handle this case.  
  Đã sửa động từ về thì hiện tại và thêm dấu chấm phẩy để nối hai mệnh đề.
- *concise*: We should handle the case where input is empty, since this function currently returns null.  
  Đã viết lại ngắn gọn hơn bằng cách đảo cấu trúc câu.
- *polite*: I believe we should also handle the case where the input is empty, as this function currently returns null.  
  Đã dùng từ 'believe' và cấu trúc câu mềm mỏng hơn để thể hiện sự tôn trọng.
- ✎ This function return → This function returns: Động từ 'return' cần thêm 's' vì chủ ngữ là số ít.
- ✎ when input is empty, I think we should → when input is empty; I think we should: Nên dùng dấu chấm phẩy để nối hai ý độc lập thay vì dấu phẩy.

## do-the-needful (Outlook)

> Please kindly do the needful and revert back to me asap.

**qwen3.5:4b** – 3.7s

- *minimal*: Please do the needful and get back to me as soon as possible.  
  Đã sửa lỗi ngữ pháp và từ vựng, giữ nguyên cấu trúc câu gốc.
- *concise*: Please act on this and reply ASAP.  
  Ngắn gọn hơn, loại bỏ các từ thừa.
- *polite*: Could you please take care of this and let me know when you can?  
  Giữ ý nghĩa nhưng dùng cách nói lịch sự và mềm mỏng hơn.
- ✎ Please kindly do the needful → Please do this: "Do the needful" là cụm từ tiếng Anh mang sắc thái cổ điển hoặc dùng trong văn cảnh hành chính, nghe hơi cứng nhắc và không tự nhiên trong giao tiếp hàng ngày.
- ✎ revert back to me → get back to me / reply to me: "Revert" có nghĩa là quay trở lại (ví dụ: quay về trạng thái trước đó), không phải "trả lời". Việc dùng "revert back" là sai ngữ pháp vì "back" đã thừa.
- ✎ asap → as soon as possible: "ASAP" thường được viết đầy đủ trong văn bản chính thức để đảm bảo tính trang trọng và rõ ràng.

## slack-indented-list (Slack)

> + Task #101 => Waiting for review.
    + Rechecked the checklist and fix related bugs => Done.
    + Rerun failed tests => Passed all.
+ Issue #202 => In progress.
    + Investigate and fix the login bugs => Done.

**qwen3.5:4b** – 4.1s

- *minimal*: + Task #101 => Waiting for review.
    + Rechecked the checklist and fixed related bugs => Done.
    + Rerun failed tests => Passed all.
+ Issue #202 => In progress.
    + Investigated and fixed login bugs => Done.  
  Đã sửa lỗi chính tả 'fix' thành 'fixed' và thêm động từ 'Investigated' cho hoàn chỉnh.
- *concise*: + Task #101: Awaiting review.
    + Checklist rechecked, bugs fixed: Done.
    + Failed tests rerun: All passed.
+ Issue #202: In progress.
    + Login bugs investigated and fixed: Done.  
  Viết ngắn gọn hơn bằng cách lược bỏ các từ thừa và dùng dấu hai chấm để rõ ràng.
- *polite*: + Task #101: Awaiting your review.
    + I have rechecked the checklist and fixed the related bugs.
    + Failed tests have been rerun; all passed.
+ Issue #202: Currently in progress.
    + Login bugs have been investigated and fixed.  
  Thêm từ ngữ lịch sự như 'I have', 'your review', 'Currently' để thể hiện sự tôn trọng.
- ✎ fix related bugs → fixed related bugs: 'Fix' ở đây cần chia thì quá khứ hoàn thành vì đã làm xong rồi.
- ✎ Investigate and fix → Investigated and fixed: Cần chia động từ theo thì quá khứ hoàn thành để đồng bộ với 'Done' ở cuối câu.

## word-client-thanks (Microsoft Word)

> Thank you for your support during the project. Hope we can cooperate in the future project.

**qwen3.5:4b** – 3.3s

- *minimal*: Thank you for your support during the project. I hope we can cooperate on future projects.  
  Chỉ sửa lỗi ngữ pháp và từ vựng, giữ nguyên cấu trúc câu.
- *concise*: Thanks for your support. Let's work together on future projects.  
  Ngắn gọn hơn, loại bỏ các từ thừa.
- *polite*: Thank you very much for your support throughout the project. I would appreciate the opportunity to collaborate with you on future endeavors.  
  Nói lịch sự và trang trọng hơn, phù hợp để thể hiện sự trân trọng.
- ✎ Hope we can cooperate in the future project. → I hope we can cooperate on future projects.: Thêm chủ ngữ 'I' cho câu hoàn chỉnh; dùng 'on' thay vì 'in' khi nói về việc hợp tác dự án; thêm 'future projects' (số nhiều) để tự nhiên hơn.
- ✎ Thank you for your support during the project. → Thank you very much for your support throughout the project.: Thêm 'very much' và 'throughout' để nhấn mạnh sự biết ơn trọn vẹn trong suốt quá trình.
