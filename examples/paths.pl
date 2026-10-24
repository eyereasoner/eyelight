% Trace links among Ada, Ben, and Cara in a three-person cycle. Every follows
% fact gives a direct reaches fact. The second rule extends a known route by
% one link only when that link ends at Ada.
%
% This deliberately restricted rule finds Ben's route through Cara to Ada. It
% does not compute every path through the cycle; in particular, it does not
% infer that Ada reaches Ada.

follows(ada, ben).
follows(ben, cara).
follows(cara, ada).

reaches(From, To) :-
    follows(From, To).
reaches(From, ada) :-
    reaches(From, Via),
    follows(Via, ada).

% Answer and Reason
?- reaches(Who, ada).
?- why((reaches(ben, ada))).

% Check
:- assertion((\+ (reaches(ada, ada)))).
:- assertion((reaches(ben, ada))).
:- assertion((reaches(cara, ada))).
:- assertion((reaches(ben, cara))).
