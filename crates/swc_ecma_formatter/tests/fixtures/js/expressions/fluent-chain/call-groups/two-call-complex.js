model = types
  .model({ something: mxSomething })
  .volatile((self) => ({ loading: false, savingStatus: "idle", undoDisabled: false, aiFocused: false, online: true }));
