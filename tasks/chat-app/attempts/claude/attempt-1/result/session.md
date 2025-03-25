# Attempt Details

## Tool
- Name: Claude
- Version: Claude Code

## Prompt Strategy
### Initial Prompt
```
/resources folder contains all the relevant context for developing Hyperware applications. Make sure to parse it, and add a configuration spec in Claude.md as a reference for yourself (for anything you might
   need). I want you to make a simple chat application, with both the backend and frontend (UI) configured. I want to be able to add people as contacts, and chat with my contacts, and be able to create 
  groupchats.
```

### Follow-up Prompts (if needed)
1. First follow-up:
   ```
   The WIT file is automatically generated based on your application. Make sure to use the built-in contacts mechanism for managing contacts.
   ```
2. Second follow-up:
   ```
   Lost track....
   ```

## Observations
I noticed a couple of pain-points for this first run.
- First, it struggled a couple of times to deduce where it needs to run `kit build` (could be in part to how the folders are structured, not sure.). 
- It had issues with figuring out how to configure the front-end to talk to the back-end. I gave it the instructions to use WebSockets
- Debugging would be really valuable if I could add the Kinode terminal errors as context.
- Context kept getting full, I think because it failed to separate out
- Data serialization/deserialization issues came up often.
- Issues with front-end talking to the backend.

## Results
Chat Application compiles and the UI loads. The only working features were adding contacts. 

## Lessons Learned
- Prompting improvements are needed. 
- Managing context better (/compact made it completely forget what we are working on.)
