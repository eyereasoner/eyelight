% Find which French cities can reach one another through the supplied road
% network. A connects(From, To) fact permits travel only in the written
% direction. A route can use one road or several roads in sequence. The
% queries find starting cities that can reach Nantes and check routes from
% Paris to Nantes and Tours, including a failed reverse-direction query.

:- table reaches/2.
connects(paris, orleans).
connects(paris, chartres).
connects(paris, amiens).
connects(orleans, blois).
connects(orleans, bourges).
connects(blois, tours).
connects(chartres, lemans).
connects(lemans, angers).
connects(lemans, tours).
connects(angers, nantes).

reaches(From, To) :-
    connects(From, To).
reaches(From, To) :-
    connects(From, Via),
    reaches(Via, To).

% Answer and Reason
?- reaches(Who, nantes).
?- reaches(paris, nantes).
?- why.
?- reaches(paris, tours).
?- why.
?- reaches(nantes, paris).

% Check
:- assertion((reaches(paris, nantes))).
:- assertion((reaches(chartres, nantes))).
:- assertion((reaches(lemans, nantes))).
:- assertion((reaches(angers, nantes))).
:- assertion((reaches(paris, tours))).
:- assertion((\+ (reaches(nantes, paris)))).
:- assertion((\+ (reaches(tours, nantes)))).
