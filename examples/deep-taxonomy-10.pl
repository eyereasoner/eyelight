% Following a classification chain 10 levels deep
%
% One individual starts in class n000000. Each level adds its next class
% and two side labels. Reaching n000010 produces a final true answer.
% This example exercises deep forward reasoning and a checkable proof chain.

% Starting classification
isa(ind, n000000).

% Completion
test(true) :- isa(ind, a2).
isa(X, a2) :- isa(X, n000010).

% Classification levels
isa(X, n000001) :- isa(X, n000000).
isa(X, i000001) :- isa(X, n000000).
isa(X, j000001) :- isa(X, n000000).
isa(X, n000002) :- isa(X, n000001).
isa(X, i000002) :- isa(X, n000001).
isa(X, j000002) :- isa(X, n000001).
isa(X, n000003) :- isa(X, n000002).
isa(X, i000003) :- isa(X, n000002).
isa(X, j000003) :- isa(X, n000002).
isa(X, n000004) :- isa(X, n000003).
isa(X, i000004) :- isa(X, n000003).
isa(X, j000004) :- isa(X, n000003).
isa(X, n000005) :- isa(X, n000004).
isa(X, i000005) :- isa(X, n000004).
isa(X, j000005) :- isa(X, n000004).
isa(X, n000006) :- isa(X, n000005).
isa(X, i000006) :- isa(X, n000005).
isa(X, j000006) :- isa(X, n000005).
isa(X, n000007) :- isa(X, n000006).
isa(X, i000007) :- isa(X, n000006).
isa(X, j000007) :- isa(X, n000006).
isa(X, n000008) :- isa(X, n000007).
isa(X, i000008) :- isa(X, n000007).
isa(X, j000008) :- isa(X, n000007).
isa(X, n000009) :- isa(X, n000008).
isa(X, i000009) :- isa(X, n000008).
isa(X, j000009) :- isa(X, n000008).
isa(X, n000010) :- isa(X, n000009).
isa(X, i000010) :- isa(X, n000009).
isa(X, j000010) :- isa(X, n000009).

% Query
?- test(Result).
