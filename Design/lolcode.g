grammar lolcode;

FILEOPEN 
	:	'#HAI' | '#hai';
FILECLOSE
	:	'#KBYE' | '#kbye';
WS         :  (' ' | '\t' | '\r' | '\n' | '\u000C')+ { $channel = HIDDEN; };
CHAR 	:	'A'..'Z' | 'a'..'z' | '0'..'9' | ',' | '.' | '"' | ':' | '?' | '!' | '%' | '/';

text	:	CHAR+;
lolcode :	FILEOPEN content FILECLOSE;
content	:	(vardef? ind_item)*;
ind_item:	comment | head | paragraph | bold | italics | list | newline | link | varuse | text;
comment	:	'#OBTW' vardef? text '#TLDR';
title 	:	'#GIMMEH TITLE' vardef? text '#OIC';
bold	:	'#GIMMEH BOLD' vardef? text '#OIC';
italics	:	'#GIMMEH ITALICS' vardef? text '#OIC';
link	:	'#GIMMEH LINX' vardef? text '#OIC';
newline	:	'#NEWLINE';
head	:	'#MAEK HEAD' vardef? head_content '#MKAY';
head_content
	:	(comment* title)?;
paragraph
	:	'#MAEK PARAGRAF' vardef? para_content '#MKAY';
para_content
	:	para_item*;
para_item
	:	bold | italics | list | newline | link | varuse | text;
list	:	'#MAEK LIST' vardef? list_content '#MKAY';
list_content
	:	list_item*;
list_item
	:	'#GIMMEH ITEM' vardef? item_content '#OIC';
item_content
	:	(bold | italics | varuse | text)*;
vardef	:	'#IHAZ' text '#ITIZ' text '#MKAY';
varuse	:	'#LEMMESEE' text '#OIC';

INVALID  :  '#' ('A'..'Z' | 'a'..'z' | '0'..'9')+;