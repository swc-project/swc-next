const state: Registry<Context, Ready extends State ? ActiveState : InactiveState> = fetchState();
