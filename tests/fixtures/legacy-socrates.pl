% Infer that Socrates is mortal from the fact that Socrates is human and the
% rule that every human is mortal. The query asks who is mortal, why asks for
% the reasoning, and assertions check the expected answer.

human(socrates).
mortal(Person) :- human(Person).

% Answer
?- mortal(Who).
% Reason
?- why.
% Check
:- assertion(mortal(socrates)).
:- assertion(aggregate_all(count, mortal(Person), 1)).
