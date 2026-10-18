# Local model benchmark – 2026-10-07

GPU idle: 15.6 W, 51 °C. Tones: minimal, concise, polite. Explanations: Vietnamese.

| Model | Valid JSON | First token (median) | Total (median) | Output tok/s | Avg / peak GPU W | Peak °C | Load |
|---|---|---|---|---|---|---|---|
| qwen3.5:4b | 9/10 | 0.19s | 2.88s | 112 | 107.4 / 110.8 | – | 54.1s |
| gemma3:4b | 10/10 | 0.16s | 2.36s | 114 | 97.8 / 111.6 | – | 12.9s |
| qwen3.5:2b | 10/10 | 0.11s | 2.63s | 134 | 107.0 / 110.0 | – | 4.6s |

## slack-urgent-pr (Slack)

> pls check my PR, it urgent

**qwen3.5:4b** – 2.7s

- *minimal*: Please check my PR, it's urgent.  
  Chỉ sửa lỗi ngữ pháp và dấu câu, giữ nguyên cấu trúc ngắn gọn của bạn.
- *concise*: PR is urgent; please check.  
  Ngắn gọn nhất, vẫn đủ ý.
- *polite*: Could you please check my PR? It's urgent.  
  Thêm từ ngữ lịch sự để mềm mỏng hơn khi yêu cầu đồng nghiệp.
- ✎ pls → Please: Viết đầy đủ 'Please' trang trọng hơn trong môi trường công việc, thay vì viết tắt 'pls'.
- ✎ it urgent → it's urgent: Cần thêm dấu chấm than hoặc từ 'is' để câu hoàn chỉnh về mặt ngữ pháp.

**gemma3:4b** – 2.4s

- *minimal*: Please check my PR, it’s urgent  
  Corrected spelling and added a possessive apostrophe.
- *concise*: Check my PR, urgent.  
  Removed 'pls' and 'it' for brevity.
- *polite*: Could you please check my PR? It’s urgent.  
  Added 'Could you please' to make the request more polite.
- ✎ pls → Please: Informal abbreviation replaced with the full word.
- ✎ it urgent → it’s urgent: Added the correct possessive form of 'it' and contracted 'is'.

**qwen3.5:2b** – 1.8s

- *minimal*: Please check my PR, it's urgent.  
  Added 'is' for grammatical correctness and separated the clauses with a comma.
- *concise*: Check PR. Urgent.  
  Removed filler words like 'my' and used imperative verbs for brevity.
- *polite*: Could you please check my PR? It's urgent.  
  Used 'could you please' to soften the request while maintaining clarity.
- ✎ pls check my PR, it urgent → Please check my PR, it's urgent.: Plural 'pls' is informal; 'it urgent' is missing a verb and an apostrophe.

## outlook-delay (Outlook)

> I want to inform that the meeting tomorrow will be delay to 3pm because customer is busy.

**qwen3.5:4b** – 4.1s

- *minimal*: I want to inform you that the meeting tomorrow will be delayed to 3pm because the customer is busy.  
  Chỉ sửa lỗi ngữ pháp và từ vựng cơ bản để giữ nguyên cấu trúc câu gốc.
- *concise*: Please note that tomorrow's meeting will be delayed to 3pm due to the customer's schedule.  
  Ngắn gọn hơn, loại bỏ từ thừa và dùng từ nối 'due to'.
- *polite*: I am writing to inform you that tomorrow's meeting will need to be rescheduled to 3pm as the customer is currently busy.  
  Thêm lời xin lỗi ngầm và dùng từ 'rescheduled' để thể hiện sự tôn trọng.
- ✎ I want to inform that → I want to inform you that: Thiếu tân ngữ 'you' sau động từ 'inform'. Trong tiếng Anh, khi nói về việc thông báo cho ai đó, cần chỉ rõ đối tượng.
- ✎ will be delay → will be delayed: Lỗi ngữ pháp: 'delay' ở đây là động từ hoặc tính từ, cần dùng dạng bị động 'delayed' với trợ động từ 'be'.
- ✎ because customer is busy → because the customer is busy: Thiếu mạo từ 'the' trước danh từ 'customer'. Cần thêm để chỉ người cụ thể này.

**gemma3:4b** – 3.2s

- *minimal*: I want to inform you that the meeting tomorrow will be delayed to 3pm because the customer is busy.  
  Corrected grammar and phrasing for a professional email.
- *concise*: Please note the meeting is now scheduled for 3pm due to customer availability.  
  Shortened and more direct wording suitable for an email.
- *polite*: Dear Mr. Tanaka, I am writing to inform you that the meeting tomorrow has been rescheduled to 3pm due to the customer’s schedule.  
  Added politeness markers and rephrased for a courteous email.
- ✎ I want to inform that → I want to inform you that: Added 'you' for clarity.
- ✎ delay to 3pm → delayed to 3pm: Corrected verb tense.
- ✎ because customer is busy → because the customer is busy: Added 'the' for correct noun usage.

**qwen3.5:2b** – 2.8s

- *minimal*: I want to inform you that the meeting tomorrow will be delayed until 3pm because the customer is busy.  
  Added 'you' for clarity and corrected 'delay' to 'delayed' (past participle) for grammatical accuracy.
- *concise*: Meeting tomorrow is delayed until 3pm due to the busy customer.  
  Removed filler words like 'I want to inform that' and combined clauses for brevity.
- *polite*: I would like to let you know that the meeting scheduled for tomorrow will be postponed until 3pm because the customer is currently busy.  
  Softened the tone with 'would like to let you know' and 'postponed' instead of 'delayed'. Added 'currently' to emphasize the present situation.
- ✎ I want to inform that the meeting tomorrow will be delay to 3pm → I want to inform you that the meeting tomorrow will be delayed until 3pm: Missing 'you' makes it sound like a direct command or unclear. 'Delay' is a noun, so it needs the past participle 'delayed'. 'To' should be 'until' when indicating a specific time.

## slack-bug (Slack)

> We has finished the task but there is some bug need to fix before release.

**qwen3.5:4b** – 3.2s

- *minimal*: We have finished the task, but there is some bug that needs to be fixed before release.  
  Chỉnh chính tả và ngữ pháp cơ bản, giữ nguyên cấu trúc câu.
- *concise*: Task done, but a bug needs fixing before release.  
  Ngắn gọn hơn, loại bỏ từ thừa.
- *polite*: We have completed the task; however, there is a bug that requires fixing before we can release it.  
  Làm mềm giọng điệu bằng cách dùng 'however', 'requires', và cấu trúc câu rõ ràng hơn.
- ✎ We has finished → We have finished: Sai chính tả: chủ ngữ 'We' đi với trợ động từ 'have', không phải 'has'.
- ✎ there is some bug need to fix → there is a bug that needs to be fixed: Sai ngữ pháp: cần thêm mạo từ 'a', động từ 'need' phải chia theo chủ ngữ và dùng cấu trúc bị động hoặc mệnh đề tương đương.

**gemma3:4b** – 3.0s

- *minimal*: We have finished the task but there is some bug that needs to be fixed before release.  
  Corrected 'has' to 'have' and added 'that' for clarity.
- *concise*: We finished the task, but a bug needs fixing before release.  
  Shortened the sentence and simplified wording.
- *polite*: We have completed the task, but there’s a bug that requires fixing before the release.  
  Used 'completed' and 'requires' for a more polite tone.
- ✎ We has finished → We have finished: Corrected subject-verb agreement.
- ✎ need to fix → needs to be fixed: Corrected verb form to agree with 'bug'.
- ✎ there is some bug → there is a bug: Added 'a' for correct article usage.

**qwen3.5:2b** – 2.8s

- *minimal*: We have finished the task, but there is a bug that needs to be fixed before release.  
  Changed 'has' to 'have' (present perfect) and split the sentence for better flow while keeping your original structure.
- *concise*: Task complete; bug fix required before release.  
  Shortened to a noun phrase, removing unnecessary words like 'the' and 'there is some' for maximum brevity.
- *polite*: We have completed the task, but we need to address a bug before releasing it.  
  Softened with 'completed' instead of 'finished', and added 'we need to address' to sound more considerate.
- ✎ We has finished → We have finished: 'has' is incorrect here; the present perfect tense requires 'have' for plural subjects like 'we'.
- ✎ there is some bug need to fix → there is a bug that needs to be fixed: 'some' should be 'a' when referring to a single item. Also, 'needs to be fixed' is more grammatically correct than 'need to fix'.

## teams-already-fine (Microsoft Teams)

> Could you review this document and share your feedback before Friday?

**qwen3.5:4b** – 2.1s

- *minimal*: Could you review this document and share your feedback before Friday?  
  
- *concise*: Please review the document and share feedback by Friday.  
  
- *polite*: Would you mind reviewing this document and sharing your feedback before Friday?  
  
- ✎ Could you review this document and share your feedback before Friday? → Could you review this document and share your feedback by Friday?: Thay 'before' bằng 'by' để chỉ thời hạn (deadline) chính xác hơn.

**gemma3:4b** – 1.7s

- *minimal*: Could you review this document and share your feedback before Friday?  
  No changes needed – the sentence is grammatically correct and clear.
- *concise*: Please review this document and provide your feedback by Friday.  
  Shortened and more direct phrasing.
- *polite*: Would you mind reviewing this document and sharing your feedback before Friday?  
  Added a polite phrase to soften the request.

**qwen3.5:2b** – 2.1s

- *minimal*: Could you review this document and share your feedback before Friday?  
  
- *concise*: Please review and share feedback by Friday.  
  Removed 'this' as it's implied; shortened the sentence structure for brevity.
- *polite*: Could you please review this document and provide your feedback before Friday?  
  Added 'please' to soften the request and made the phrasing slightly more courteous while keeping the original structure intact.
- ✎ Could you review this document and share your feedback before Friday? → Could you please review this document and provide your feedback before Friday?: Added 'please' to make the request sound more polite, which is especially important in professional contexts like Microsoft Teams.

## vi-weekly-report (Slack)

> Anh ơi em gửi báo cáo tuần rồi nhé, anh xem giúp em với

**qwen3.5:4b** – 2.9s

- *minimal*: Anh ơi, em gửi báo cáo tuần rồi nhé, anh xem giúp em với.  
  
- *concise*: Em đã gửi báo cáo tuần rồi, anh xem giúp em nhé.  
  
- *polite*: Anh ơi, em vừa gửi báo cáo tuần rồi. Anh có thể dành chút thời gian xem giúp em được không ạ?  
  
- ✎ báo cáo tuần rồi nhé → báo cáo tuần rồi: Thêm 'rồi' ở cuối câu khiến nó nghe như một lời nói chuyện hàng ngày hơn là một thông báo rõ ràng.
- ✎ anh xem giúp em với → anh xem giúp em nhé / anh có thể xem giúp em được không ạ?: 'Với' trong ngữ cảnh này nghe hơi thiếu tự nhiên; dùng 'nhé' hoặc câu hỏi lịch sự sẽ tốt hơn.

**gemma3:4b** – 1.7s

- *minimal*: Can you please review the weekly report I sent, please?  
  Fixed informal phrasing and added a polite request.
- *concise*: Please review the weekly report.  
  Shortened the request for directness.
- *polite*: Could you please take a look at the weekly report I sent? Thank you.  
  Added a polite phrase and a thank you for a more courteous tone.

**qwen3.5:2b** – 2.6s

- *minimal*: Anh ơi, em gửi báo cáo tuần rồi nhé, anh xem giúp em với.  
  
- *concise*: Anh ơi, gửi báo cáo tuần rồi. Anh xem giúp em.  
  Removed 'nhé' and 'với' for brevity while keeping the casual tone.
- *polite*: Anh ơi, em đã gửi báo cáo tuần rồi nhé, anh xem giúp em được không?  
  Added a polite question structure to soften the request and show consideration.
- ✎ Anh ơi em gửi báo cáo tuần rồi nhé → Anh ơi, em đã gửi báo cáo tuần rồi nhé: Added 'đã' (already) to clarify that the report was sent in the past, making the statement more natural and precise.
- ✎ nhé, anh xem giúp em với → nhé, anh xem giúp em được không?: Changed 'với' (with) to a polite question structure ('được không?') to soften the request and show consideration.

## mixed-deploy (Slack)

> em đã fix bug login rồi, nhưng chưa deploy lên staging được vì server down

**qwen3.5:4b** – 3.3s

- *minimal*: I've fixed the login bug, but haven't deployed to staging yet because the server is down.  
  
- *concise*: Login bug fixed; can't deploy to staging due to server outage.  
  
- *polite*: I've managed to fix the login bug, but I'm unable to deploy to staging right now as the server is down.  
  
- ✎ em đã fix bug login rồi → I've fixed the login bug: Chuyển từ tiếng Việt sang tiếng Anh tự nhiên hơn, thêm chủ ngữ và động từ hoàn thành.
- ✎ chưa deploy lên staging được → haven't deployed to staging yet: Sử dụng cấu trúc hiện tại hoàn thành với 'yet' để diễn tả hành động chưa thực hiện được.
- ✎ vì server down → because the server is down: Thêm chủ ngữ và động từ liên kết để câu hoàn chỉnh, thay vì chỉ dùng 'down' như một tính từ đứng độc lập.

**gemma3:4b** – 2.0s

- *minimal*: I’ve fixed the login bug, but haven’t been able to deploy to staging because the server is down.  
  Fixed grammar and phrasing for a more natural English flow.
- *concise*: Login bug fixed, but staging deployment failed due to server downtime.  
  Shortest possible version, removing unnecessary words.
- *polite*: I’ve resolved the login bug, however, I’m unable to deploy to staging at the moment due to server downtime.  Could you please investigate?  
  Added a polite request for assistance.

**qwen3.5:2b** – 2.5s

- *minimal*: Em đã fix bug login rồi, nhưng chưa deploy lên staging được vì server down.  
  
- *concise*: Login bug fixed locally; deployment blocked due to server outage.  
  Shortened the sentence structure while keeping all key information intact.
- *polite*: I've already resolved the login issue, but we can't deploy it yet because the server is down.  
  Added 'I've' and 'we can't' to soften the tone while maintaining clarity.
- ✎ em đã fix bug login rồi → I've already resolved the login issue: Changed 'em' to 'I've' for a more professional tone, and 'bug login' to 'login issue' for clarity.
- ✎ chưa deploy lên staging được → we can't deploy it yet: Replaced the colloquial phrase with a clearer expression of the current blockage.

## email-late-reply (Outlook)

> Sorry for late reply, I was busy with other project so I didn't see your email.

**qwen3.5:4b** – 2.1s – ❌ tones minimal

- *minimal*: Sorry for the late reply. I was busy with another project, so I didn't see your email.  
  Chỉ sửa lỗi chính tả và ngữ pháp nhỏ, giữ nguyên cấu trúc câu gốc.
- ✎ Sorry for late reply → Sorry for the late reply: Cần thêm từ 'the' để chỉ rõ một lần trả lời muộn.
- ✎ I was busy with other project → I was busy with another project: Danh từ 'project' cần thêm mạo từ hoặc dùng 'another' thay vì 'other' ở đây.

**gemma3:4b** – 5.4s

- *minimal*: Sorry for the late reply; I was occupied with another project and missed your email.  
  Fixed awkward phrasing and added a semicolon for clarity.
- *concise*: Apologies for the delayed response; I was working on a different project.  
  Shortened the sentence for directness.
- *polite*: Please accept my apologies for the late reply. I was fully engaged with another project and unfortunately didn’t see your email until now.  
  Added polite phrasing and emphasized the delay.
- ✎ Sorry for late reply, I was busy with other project so I didn't see your email. → Sorry for the late reply; I was occupied with another project and missed your email.: Replaced 'so' with a semicolon and rephrased for better flow.

**qwen3.5:2b** – 3.1s

- *minimal*: Sorry for the late reply. I was busy with another project, so I missed your email.  
  Added 'the' before 'late' and 'another' to make it grammatically complete; removed 'so I didn't see' for clarity while keeping your structure.
- *concise*: Late reply. Busy with another project, missed your email.  
  Removed filler words like 'Sorry for', 'I was', and 'so I didn't' to make it as short as possible while remaining clear.
- *polite*: My apologies for the late response. I got caught up on another project, so I missed your email.  
  Used 'My apologies' and 'got caught up' to soften the tone while keeping the original meaning intact.
- ✎ Sorry for late reply → Sorry for the late reply: Added 'the' after 'late' because you need a noun phrase to complete the prepositional phrase.
- ✎ I was busy with other project so I didn't see your email. → I was busy with another project, so I missed your email.: Changed 'other' to 'another' (singular) because you are referring to a specific single project. Also changed 'didn't see' to 'missed' for better flow.

## code-review (Google Chrome)

> This function return null when input is empty, I think we should handle this case.

**qwen3.5:4b** – 2.5s

- *minimal*: This function returns null when input is empty. I think we should handle this case.  
  
- *concise*: Handle the null case when input is empty.  
  
- *polite*: I think we might want to handle this case, since the function returns null when the input is empty.  
  
- ✎ return → returns: Chức động từ 'return' cần thêm đuôi 's' vì chủ ngữ là 'This function' (số ít).
- ✎ input is empty → the input is empty: Thêm mạo từ 'the' để chỉ rõ đối tượng cụ thể.

**gemma3:4b** – 1.8s

- *minimal*: This function returns null when the input is empty; I think we should handle this case.  
  Corrected grammar and phrasing for a more natural flow.
- *concise*: This function returns null for empty input, we should handle it.  
  Shortened the sentence for clarity.
- *polite*: This function returns null when the input is empty. I believe it would be beneficial to address this case.  
  Added a polite phrasing to soften the suggestion.

**qwen3.5:2b** – 2.4s

- *minimal*: This function returns null when the input is empty; we should handle this case.  
  Fixed the verb 'return' to 'returns' and added 'the' before 'input' for grammatical correctness while keeping your original structure.
- *concise*: Returns null on empty input; handle this case.  
  Removed unnecessary words like 'This function', 'when', and 'we should' to make it the shortest possible version while remaining clear.
- *polite*: This function returns null when the input is empty; we should consider handling this case.  
  Softened the statement with 'consider handling' and added a polite tone to make it suitable for collaborative discussion.
- ✎ This function return null when input is empty → This function returns null when the input is empty: The verb 'return' needs to be in the third person singular form 'returns' because 'function' is a singular noun.

## do-the-needful (Outlook)

> Please kindly do the needful and revert back to me asap.

**qwen3.5:4b** – 3.7s

- *minimal*: Please kindly do the needful and revert back to me asap.  
  
- *concise*: Please do this and get back to me ASAP.  
  Giảm bớt từ thừa và dùng từ thông dụng hơn trong email.
- *polite*: Could you please take care of this and let me know when you're done?  
  Làm mềm giọng điệu bằng cách hỏi thay vì ra lệnh, vẫn giữ sự khẩn cấp.
- ✎ do the needful → take care of this / handle this: 'Do the needful' là cụm từ tiếng Anh mang sắc thái Ấn Độ, nghe rất lạ với người bản xứ.
- ✎ revert back → get back to me / let me know: 'Revert' đã không còn nghĩa là 'trả lời' trong tiếng Anh hiện đại; thêm 'back' sau 'revert' cũng sai ngữ pháp.
- ✎ asap → ASAP (giữ nguyên hoặc thay bằng 'urgently'): 'Asap' là viết tắt, nên viết hoa toàn bộ trong văn bản chính thức; tuy nhiên nó vẫn được chấp nhận rộng rãi.

**gemma3:4b** – 2.4s

- *minimal*: Please do the needful and revert back to me asap.  
  Fixed the slightly awkward phrasing 'do the needful' for a more standard English expression.
- *concise*: Please take care of this and let me know as soon as possible.  
  Shortened the sentence and replaced 'do the needful' with a clearer alternative.
- *polite*: Could you please take care of this and revert back to me as soon as possible?  
  Added 'Could you please' to make the request more courteous.
- ✎ Please kindly do the needful → Please do the needful: Removed the unnecessary 'kindly' which adds no value.

**qwen3.5:2b** – 3.0s

- *minimal*: Please do the needful and revert back to me ASAP.  
  Fixed 'kindly' to 'do the needful' for a more professional tone while keeping your original structure.
- *concise*: Please do the needful and revert ASAP.  
  Removed 'back to me' to make it shorter, assuming context implies you are the recipient.
- *polite*: Could you please do the needful and revert back to me as soon as possible?  
  Softened with 'could you please' and 'as soon as possible' for a courteous tone.
- ✎ Please kindly do the needful → Please do the needful: 'Kindly' is redundant with 'do the needful'; removing it makes the sentence more concise and professional.
- ✎ revert back to me → revert back to me: This phrasing is acceptable in business contexts, though 'revert' alone might be slightly more direct depending on your relationship with the recipient.
- ✎ asap → ASAP: In formal emails, capitalizing 'ASAP' is often preferred for consistency and professionalism.

## word-client-thanks (Microsoft Word)

> Thank you for your support during the project. Hope we can cooperate in the future project.

**qwen3.5:4b** – 2.9s

- *minimal*: Thank you for your support during the project. I hope we can cooperate in future projects.  
  
- *concise*: Thanks for your support. Looking forward to future cooperation.  
  
- *polite*: Thank you very much for your support throughout the project. We would appreciate the opportunity to collaborate on future projects.  
  
- ✎ Hope we can cooperate in the future project. → I hope we can cooperate in future projects.: Thêm chủ ngữ 'I' để câu đầy đủ; thêm 's' ở 'projects' vì nói chung là nhiều dự án hơn một.
- ✎ cooperate → collaborate: 'Collaborate' nghe tự nhiên và chuyên nghiệp hơn trong ngữ cảnh làm việc nhóm so với 'cooperate'.

**gemma3:4b** – 1.9s

- *minimal*: Thank you for your support during the project. I hope we can cooperate on future projects.  
  Fixed phrasing for clarity and formality.
- *concise*: Thank you for your support. Let’s collaborate on future projects.  
  Shortened and simplified the expression.
- *polite*: Thank you again for your support during the project. I would be delighted to collaborate with you on future projects.  
  Added a more courteous phrasing.

**qwen3.5:2b** – 2.5s

- *minimal*: Thank you for your support during the project. We hope to cooperate on future projects.  
  Added 'We' for natural subject-verb agreement and corrected the slightly awkward phrasing of 'future project' to 'future projects'.
- *concise*: Thanks for your support. Hope to collaborate on future projects.  
  Removed filler words like 'during' and 'hope we can', making it shorter while keeping the meaning clear.
- *polite*: Thank you for your continued support during this project. We look forward to collaborating on future initiatives.  
  Used 'continued' and 'collaborating on future initiatives' to soften the tone while maintaining professionalism.
- ✎ Hope we can cooperate in the future project. → We hope to collaborate on future projects.: The original sentence was slightly awkward. 'Cooperate' is often used as a verb with an object (e.g., 'cooperate with'), so adding 'with' and changing to 'collaborate' makes it more natural.
