for (((/* prettier-ignore */ name  in object as boolean)); false;);
for ((/* prettier-ignore */ name  in object satisfies boolean); false;);
for ((((/* prettier-ignore */ value = left in right as boolean))); false;) {}
for ((((/* prettier-ignore */ value = left in right satisfies boolean))); false;) {}
for (((/* prettier-ignore */ value = left in right as boolean)), second = true; false;) {}
for (value = false, ((/* prettier-ignore */ second = left in right satisfies boolean)); false;) {}
for (let picked = (((/* prettier-ignore */ value = left in right as boolean))); false;) {}
