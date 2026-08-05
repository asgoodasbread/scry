/// The instruction sent alongside the screenshot.
pub fn user_prompt() -> &'static str {
    "Identify all the questions in this image and answer them. Return ONLY the answer, \
     as concisely as possible. Do not include any reasoning, conversational filler, \
     or introductory text. Please give the answer index appropriate to the question which is being \
     answered."
}
