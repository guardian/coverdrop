// As per https://github.com/guardian/coverdrop/issues/2680 we discovered that including these strings as .md
// can cause issues with the final build process where they are removed from the final artifact. As a temporary
// solution we include them as statically included strings.

// swiftlint:disable line_length

let helpMarkdownCraftMessage = """
# Compose your first message

BLOCKQUOTE
We receive hundreds of tips a day. The messages that stand out are clear, detailed and fact-focused.
Paul Lewis
Head of investigations

Messages are most credible when they are packed with facts, refer to evidence for those facts, and provide some context for how you know this.

DIVIDER

## Include what’s safe

### Facts

Specifics are critical - such as names, locations and dates.

EXAMPLE
I work at ~government department ‘Department name’~ in the ~procurement section~.

### Clearly state the nature of your tip

Be concise and focus on what you know, rather than what you think or suspect.

EXAMPLE
I have discovered ~evidence of a senior official's involvement in corruption and bribery~. ‘Name’, the head of our section, has received $150,000 in under-the-radar payments.

### Evidence

Tell us how you know what you know, and how we might verify it.

EXAMPLE
I have ~photographic evidence of transactions~ from the past two years and access to ~further documents~ that show these were a quid pro quo for awarding contracts to multinational company ‘Company name’.

Our journalists appreciate that sharing information can be intimidating. You should only share what is best for your safety. However, vague messages are rarely actionable.

SPACE

BLOCKQUOTE
You and your information will be treated with respect. Good sources are extremely important to us, and protecting their anonymity is a top priority.
Nick Hopkins
Head of news

DIVIDER

## Try to avoid…

### Being vague

Journalists need facts. If you don't explain the who, what, how and when, a reporter is unlikely to spend time trying to work out what you're talking about. Something like the following is unlikely to get a response:

EXAMPLE
I have ~some information~, would you like to know more? ~Someone I know~ is doing ~something illegal~.

### Anonymity

The Secure Messaging system doesn't let us know who you are. If you don't want to be identified as a result of our reporting, we will do everything we can to prevent that from happening. But the credibility of a story tip relies on the credibility of its source. If you can share why you know what you know, it's far more likely to be taken seriously.

EXAMPLE
I know about a criminal abuse of power but ~I can't tell you how I know it~.

### Time pressure

Reply times can vary. It could take several days. If it can’t wait, consider faster but less secure methods such as email or telephone.

EXAMPLE
Make sure you reply to me on here by the ~end of today~.

DIVIDER

BUTTON
Source protection
Read more
button_source_protection
"""

let helpMarkdownFaq = """
# Frequently asked questions

## General

### Who is Secure Messaging for?

We built the Guardian Secure Messaging service to allow readers to contact our journalists directly about news stories that we are covering or should be covering. If you know about something that should be reported, tell us using Secure Messaging and we will handle your tip confidentially and responsibly.

Unlike other apps that require technical skills to ensure security, Secure Messaging is designed for anyone to use with ease. All you need is to remember your passphrase. Whether you're a PA, nurse, data scientist, police officer, supermarket worker, public servant, intern, or CEO, this app is for you. It’s made for people who are witnessing injustice and want the Guardian’s help to bring it to light.

DIVIDER

### Who is it not for?

The Secure Messaging service is purely for story tips. Don’t use it for customer service enquiries, letters to the editor, article submissions, complaints, etc. Such messages will be ignored. Please see theguardian.com/contact-us for advice on how to get in touch about those things.

DIVIDER

### How does Secure Messaging work?

We have a dedicated help page on how Secure Messaging works. It explains how the system is designed to protect your identity and keep your messages secret.

BUTTON
How Secure Messaging works
Read more
button_how_secure_messaging_work

DIVIDER

### Is it really anonymous? I’m signed in to the Guardian app.

You don’t have to be signed in to the Guardian app to use Secure Messaging, but Secure Messaging is anonymous even if you are signed in. The Secure Messaging part of the app only knows the content of your messages. It has no access to information that the overall Guardian app may have – not your email, name, location, reading history, etc.

As a consequence no identifying information accompanies the messages received by our journalists. We can’t even tell which devices sent us real messages. We know nothing more than what the senders tell us.

BUTTON
Privacy policy
Read more
button_privacy_policy

DIVIDER

## Interaction with journalists

### I don't know who to speak to. Who should I contact?

If you don’t know which journalist might specialise in the matter you wish to raise, you could browse our existing journalism on related subjects and see what names appear next to those articles. If you’re still not sure, pick a team such as “Environment team” or “Investigations team”. Those messages will go to the duty editor for that team. If they want to follow up they may do so themselves or they may direct you to a specific reporter.

DIVIDER

### What does 'Pending' mean? Why aren't my messages sent immediately?

The pending message you see is part of a sophisticated system to keep your actions hidden. Your real message is delayed because it will be sent when your phone would normally send a decoy “cover” message. This means the time you use the app cannot be linked to when a message is sent. This disguise technique is one of the many ways we keep you and your messages secret.

To ensure real messages are indistinguishable from the “cover” messages sent by the Guardian app, both messages must be sent at similar times. Otherwise, activity patterns could reveal when real messages are sent, potentially identifying a source.

DIVIDER

### Who gets to see my messages?

If you select a specific journalist as a recipient, your message is routed directly to them. If you select a team, your message will go to the editor or editors in charge of that team. Secure Messages are end-to-end encrypted and can’t be deciphered by anyone else.

Under some circumstances a substitute journalist will act on behalf of the designated recipient – for example if a reporter is briefly unavailable to respond to an ongoing conversation.

Similarly, if someone using our apps should try to abuse the system, the recipient journalist may ask a colleague to take over. In both cases the addressee would temporarily transfer the means to decrypt their correspondence to a trusted colleague who is experienced in dealing with highly confidential material.

DIVIDER

### How can I be sure that my messages are really going to the Guardian?

We use digital security keys to secure the content of your messages. The Guardian's main key is held in a secure location and you can verify that the app is identifying the correct one as follows:
- Go back to the About Secure Messaging screen.
- Scroll down to the bottom.
- You'll see the word "root:" followed by a code identifying the key that the app is currently using.
- Compare that code to the "Secure Messaging Digest" code that is published in the small print on page 2 of the Guardian newspaper in print.

DIVIDER

### How long does it take to get a reply?

We can’t promise we’ll reply to anyone. Even when we do, it can take a while. For security reasons, your messages are not sent immediately and aren’t received immediately by us after being sent. Similarly, replies from journalists are neither sent nor received immediately. So even if a reporter replies the moment they receive a message, you may not see that reply for hours. And of course each reporter doesn’t work 24 hours a day or every day, so it could take days.

BUTTON
What to expect as a reply
Read more
button_what_to_expect_as_a_reply

DIVIDER

### Why can't I talk to more than one person at a time?

Messages must be infrequent to keep “cover” effective without increasing data use. That’s why only one conversation is allowed at a time. This also helps prevent spam and abuse.

DIVIDER

### How do I choose to speak with a different journalist or team?

There are two ways to do this. You can delete your old conversation to pick someone new to correspond with. This retains your current passphrase. Or you can choose to 'Get started' from the first Secure Messaging page, and set up a new message vault with a new passphrase.

DIVIDER

## Troubleshooting / technical

### How do I report a technical problem?

You can report problems with Secure Messaging by going back into the main app, selecting Settings and then Help. However, please note that unlike your communication using Secure Messaging, bug reports are not anonymous: they’re sent from your own email address and can include details about the phone you’re using and your Guardian subscription (if you have one). Your bug report shouldn't discuss the content of your Secure Message.

Keep in mind that bug reports are sent via email. If someone gains access to your emails, they could see that you’ve attempted to use Secure Messaging. This means your use of the system is not secret.

DIVIDER

### How do I report security issues?

To let us know about security issues with CoverDrop, the technology platform that powers Secure Messaging, we would prefer you email us under encryption. Please write to coverdrop.security@theguardian.com and use the public key with the fingerprint: 853F 7EFF 332F DA4F 66ED CD83 3CA0 0C56 15FA 4518

DIVIDER

### I need to send a document. What should I do?

Contact us via Secure Messaging first, and say you have documents to share. A reporter will discuss with you how best to do that safely.

DIVIDER

### I forgot my passphrase. What should I do?

We cannot reset your passphrase or provide reminders, as this would weaken security. If you lose it, please create a new secure message vault and start a new conversation.

DIVIDER

### What if someone compels me to show them my Secure Messaging vault?

All Guardian apps automatically include an encrypted Secure Messaging vault. Whether a vault was created automatically or set up by you, it behaves identically: it's encrypted the same way, saved at the same times, and it always stays the same size.

This means that neither the app itself, nor someone with access to your smartphone, can tell the difference. If you were to enter an incorrect passphrase, the app would not know if you actually have a "real" message vault (which can be unlocked with a different passphrase) or if the vault was created automatically with a random passphrase that was never shown. The app responds to an incorrect passphrase by reporting, truthfully, that the passphrase does not work either because it's wrong or because you didn't create a vault at all.

Consequently you can plausibly deny that you have ever set up a vault by saying "I do not have a passphrase and I never did".
"""

let helpMarkdownHowSecureMessagingWorks = """
# How Secure Messaging works

Secure Messaging is designed to let people communicate with journalists without leaving evidence that they have done so.

This protects sources from being identified either by the organisations they want to inform us about, or by other parties – even those with sophisticated surveillance capabilities.

It works by hiding real messages among automatically generated decoy messages.

DIVIDER

### Every Guardian app user provides cover for our sources

In order to receive content, news apps exchange data with the app providers’ servers. With the introduction of the Secure Messaging system, the Guardian app automatically inserts tiny quantities of specially encrypted information along with that regular network activity. Every Guardian app does this, whether they’re being used for Secure Messaging or not. These automatic “messages” don’t actually contain real information. But they provide what we call “cover” for when real messages do get sent to and from our journalists.

### Real messages look no different

To tip off the Guardian about a news story, you can create a Secure Messaging conversation in the Guardian mobile app. When you hit “send”, some of the “cover” data will be replaced with the text of your Secure Message. These real messages are encrypted and stored in exactly the same way as the “cover” messages. They are the same size, and they get transmitted at the same times. This makes them indistinguishable from the cover messages generated by everyone using the Guardian app.

However, once the Guardian receives both cover and real messages, our Secure Messaging system removes an outer layer of encryption to identify which are the real messages and which journalist or team they are for. The real messages are then delivered to our journalists, who decrypt the inner layer of encryption to read the content.

### Our replies are also under cover

Replies from journalists are similarly hidden in “cover” traffic from the Guardian to our mobile apps. But genuine replies can only be identified and decrypted in the app used by the person who started the conversation.

Consequently, confidential communication with the Guardian looks exactly the same as completely normal use of the Guardian mobile app to view our content.

### Sources can say they're just readers

If someone were to be accused of being a source, their Guardian app would not look different from any other Guardian app. Millions of people have the Guardian app installed simply to read our content, so having the app doesn’t prove anything.

DIVIDER

BUTTON
Frequently asked questions
Read more
button_faq
"""

let helpMarkdownKeepingPassphrasesSafe = """
# Keeping passphrases safe

## What is a passphrase?

Your Secure Messaging passphrase is a set of words known only by you. It grants you access to your conversation with the Guardian on this mobile device. You will need to enter your passphrase to see replies from us and to send follow-up messages.

Our software creates your passphrase, but we do not store it. If you forget it, we can't get you back in to your conversation.

PASSPHRASE_BOXES apple waterfall diamond

### Should I write my passphrase down somewhere safe?

We recommend you record your passphrase somewhere it won't be found or in a password manager.

SPACE

### Can I screenshot my passphrase?

This is not secure. If confidentiality isn’t a concern, it might work for you.

SPACE

### Can I just memorise my passphrase?

You can. But if you forget it, you will not be able to continue a conversation.

DIVIDER

## Remembering passphrases

### What if I do forget my passphrase?

Only you have your passphrase; no one else. Our systems cannot remind you. This is to prevent even the most sophisticated adversary from gaining access to your communication.

If you lose your passphrase you will need to start a new conversation. For security reasons, when you start a new conversation, your old conversation will be deleted from this device.

If you start a new conversation with the same journalist as before, they will not know you're the same person. But you may be able to remind the journalist by referring to details shared in the prior conversation.

DIVIDER

## What if someone else learns my passphrase?

Your passphrase only works on the phone on which you set it. No one can read your messages on a different phone, even if they somehow knew your passphrase.

If you suspect someone knows your passphrase and may gain access to your phone, you can delete your old conversations and reset the passphrase by starting a new conversation.

When you start a new conversation you will be required to set a new passphrase. At this point your old message vault will be wiped, even if you don't write any new messages.
"""

let helpMarkdownPrivacyPolicy = """
# Secure Messaging privacy policy

The Secure Messaging software in the Guardian app is currently in a testing phase. At this stage the feature does not have a distinct privacy policy to that which we publish for the rest of the Guardian app.

### All Secure Messaging users are anonymous

This feature is designed to be used anonymously. As a consequence, messages sent to our journalists using Secure Messaging contain no accompanying data about who sent that message, from where, and from what kind of device. We can't even tell exactly when you wrote a message.

If you are signed in to the Guardian app, that information is not known by the Secure Messaging feature. Similarly, the rest of the Guardian app on your mobile device does not know if you are sending messages to the Guardian using Secure Messaging. The Secure Messaging App cannot discover anything about you that you don't choose to tell us.

Your correspondence with our journalists is held in an encrypted vault on your phone. This vault is secured by a passphrase known only to you. Messages disappear automatically after 14 days.

BUTTON
Passphrase security
Read more
button_help_keeping_passphrase_safe

At the point at which you send any information to us, the Guardian’s Editorial Code applies. See theguardian.com/editorial-code

Last updated March 2025
"""

let helpMarkdownReplyExpectations = """
# What to expect as a reply

We may not reply to all messages. If we do reply, you can expect the following:

- Once we enter into correspondence with you, that conversation and any reporting that may come of it falls under the Guardian’s Editorial Code. See theguardian.com/editorial-code
- If you messaged a team rather than an individual, a reporter or editor will identify themself in the reply.
- They may ask you for more details about your tip.
- They may ask for more details about how you came by this information. Be assured that disclosing information to a Guardian journalist that could identify you does not mean they will reveal that information to anyone else. We are committed to protecting our sources.
- They may ask if you have or know of any documentary proof of what you told them.
- If you don't want to be identified as a source, they may want to discuss if any of the evidence you have could in theory be traced back to you.
- If appropriate they may suggest a more direct conversation such as a phone call or similar. To achieve this they may provide personal contact details.

DIVIDER

Sometimes you may get a reply from a reporter saying they're unable to help. This might be because:

- The reporter is unable to follow this up at this time due to absence or other commitments. They may suggest you contact a different journalist, or try again in future.
- The reporter doesn't cover the issues you have raised, or in the geographic region that they pertain to. They may suggest you contact someone else.
- The reporter believes the issue may not be a matter of public interest. For example if some person has committed some misconduct, we won't necessarily report on it unless we believe it has significant implications for the general public and that they have the right to know about.
"""

let helpMarkdownSourceProtection = """
# Source protection

BLOCKQUOTE
Journalists have a moral obligation to protect confidential sources of information
The Guardian's editorial code of practice
theguardian.com/editorial-code

Whistleblowers and witnesses to wrongdoings are essential to holding power to account. But this can come with risks to the people who speak out. This is why sources often ask to remain anonymous.

DIVIDER

When a Guardian journalist agrees not to disclose the identity of a source, the journalist will attempt to frame the articles they write in ways that don't reveal who they have been speaking to. Even when facing litigation, journalists have the right to protect the identity of their sources who acted in the public interest.

Our Secure Messaging system was created to help protect sources’ anonymity:

- A Guardian mobile app that has been used to message the Guardian looks the same as a Guardian app that has only been used to view Guardian content.
- The data transmitted to and from the Guardian looks the same for Secure Messaging users as it does for all other Guardian app users.
- The information the Guardian receives from Secure Messaging sources comes with no accompanying information about who sent it or from where.

So we know nothing more about a Secure Messaging source than what a source chooses to tell us.

DIVIDER

This doesn’t necessarily mean that all sources remain anonymous to the reporter themself. That scenario is in fact rare. More commonly, a confidential source will tell a reporter who they are, or at least provide some identifying information, in order to explain how they know about the subject they have raised, and to show that their claims are credible. It can also help a journalist corroborate their source’s allegations without unwittingly revealing clues about who that source is.
"""

let helpMarkdownWhyWeMadeSecureMessaging = """
# Why we made Secure Messaging

The Guardian’s Secure Messaging tool allows readers like you to tip off our journalists about newsworthy stories.

It’s a unique new system based on research by the University of Cambridge’s Department of Computer Science and Technology.

The Cambridge researchers set out to solve challenges faced by those who want to share information about wrongdoing with news organisations. These sources act in the public interest, but that doesn’t mean they seek attention, in fact, they may fear scrutiny or even retaliation. They need a safe communication method that doesn't reveal what is said, or who is saying it.

DIVIDER

BLOCKQUOTE
Democracy demands information.
Antoine Deltour
Source of the LuxLeaks

SPACE

Many of the traditional methods are just not secure enough. And the best of them still have problems. They may require advance knowledge of private contact details. They can be difficult to use. And they can involve software that itself could raise suspicion.

Secure Messaging is designed to tackle these problems by introducing a system for contacting journalists into the normal Guardian app. It's easy to use and extremely secure. And, critically, every copy of the Guardian app behaves the same way, whether it's being used for Secure Messaging or not. This means sources look no different to millions of other users.

DIVIDER

BUTTON
How Secure Messaging works
Read more
button_how_secure_messaging_works
"""

// swiftlint:enable line_length
