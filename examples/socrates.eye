% Socrates and mortality
%
% Socrates is human, and every human in this model is mortal.
% Eyelight derives his mortality and returns it as a query answer.
% This introduces facts, a rule, and a named query variable.

human(socrates).
mortal(Person) :- human(Person).

?- mortal(Who).
