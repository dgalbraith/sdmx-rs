<details>
<summary>XSD contract: <code>AnnotationType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="AnnotationType">
		<xs:annotation>
			<xs:documentation>AnnotationType provides for non-documentation notes and annotations to be embedded in data and structure messages. It provides optional fields for providing a title, a type description, a URI, and the text of the annotation.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="AnnotationTitle" type="xs:string" minOccurs="0">
				<xs:annotation>
					<xs:documentation>AnnotationTitle provides a title for the annotation.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="AnnotationType" type="xs:string" minOccurs="0">
				<xs:annotation>
					<xs:documentation>AnnotationType is used to distinguish between annotations designed to support various uses. The types are not enumerated, as these can be specified by the user or creator of the annotations. The definitions and use of annotation types should be documented by their creator.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="AnnotationURL" type="AnnotationURLType" minOccurs="0" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>AnnotationURL is a URI - typically a URL - which points to an external resource which may contain or supplement the annotation. These can be localised by indicating a language for the resource. If a language is not specified, the resource is assumed to not be localised. If a specific behaviour is desired, an annotation type should be defined which specifies the use of this field more exactly.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="AnnotationText" type="TextType" minOccurs="0" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>AnnotationText holds a language-specific string containing the text of the annotation.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="AnnotationValue" type="xs:string" minOccurs="0">
				<xs:annotation>
					<xs:documentation>AnnotationValue holds a non-localised value for the annotation.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:sequence>
		<xs:attribute name="id" type="xs:string" use="optional">
			<xs:annotation>
				<xs:documentation>The id attribute provides a non-standard identification of an annotation. It can be used to disambiguate annotations.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
