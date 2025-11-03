Annotations 

Introduction

	Annotations are a core element in lex, but a metadata one. They provide , not only a way for authors and colaborators to register non content related information, but the right hooks for tooling to build on top of lex, such as a commenting system before publishing.

	As such they provide labels (a way to identify the annotation) and parameters (a way to provide structured metadata for tooling).


	Core features:

	- Annotations have an optional labels [./labels.lex] and parameters [./parameters.lex]
	- Labels are mandatory, parameters are optional.
	- Annotations have optional content: which can be the single line shortcut or the regular content conatainer form, which allows all elements but sessions to be part (including nesting). While not prohibited, annotations should not contain other annotations as their content as the semantic meaning would be ... why bother? 

Syntax Forms:

	Marker form (no content):
		:: label ::
		:: label params ::

	Single-line form (inline text):
		:: label :: text content here
		:: label params :: text content here

	Block form (indented content - note TWO closing :: markers):
		:: label ::
		    indented paragraph or list
		::


Content

	Can be empty (marker form - the label itself carries meaning)
	Forms:
	- Inline text (single-line form)
	- Block content (paragraphs/lists, but NOT sessions or nested annotations)
	
Attachment
 
	Annotations are not part of the document per se, but metadata about part(s) of the lex document. 
		hence annotations are attached to these elements at later stages. The rules for attaching annotations are: 
	- Previous element: annotations attach to the previous non blank line element in the same container.
	- Document level: when annotations are the first document elements , up to a blank line or any other element
	- Parent: if no such prev element exists, it's attached to it's parent element (the container in which it appears)
	
Examples:

:: note :: Important information
:: warning severity=high :: Check this carefully
:: debug :: (marker form, no content)
:: type=python :: (params-only, no label)