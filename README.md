# rforth

## learning (or not) Rust, while vibe coding

The only file edited manually is this README; all other files—including documentation and examples—were generated by the Copilot Chat agent (most example suggestions came from the AI). I manually corrected the factorial example to use `<` instead of `<=`.

I have not learned rust before, and forgot what I learnt from https://thinking-forth.sourceforge.net/ (a great programming book, with incredibly modern ideas for a `84 book)

## Models

I used a GitHub Copilot Pro plan ($10/month at the time).

I started with Gemini 2.1 preview, but it stopped responding and returned 500 errors after restarting VS Code. It did a good job. I then switched to o4-mini (preview), which also produced good results but at a slower pace.

## Chat

I used "Chat: Export chat..." in VS Code and saved the result as `chat/day1.json`.
Day1 is  a ~4 hours session while watching TV

## Random Thoughts

- This was my second attempt; the first became messy and regressed over time.
- Incremental progress in small steps works better.
- Larger steps than typical TDD can be effective (e.g. implementing all arithmetic ops at once).
- Clear requirements and verification remain essential; AI can generate tests, but QA is still required.
- Minimal Rust understanding gained; a thorough code review is needed.

## todo
- add `execute` and more of the suggested examples