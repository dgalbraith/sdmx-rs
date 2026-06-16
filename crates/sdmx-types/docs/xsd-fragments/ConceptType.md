<details>
<summary>XSD contract: <code>ConceptType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="ConceptType">
		<xs:annotation>
			<xs:documentation>ConceptType describes the details of a concept. A concept is defined as a unit of knowledge created by a unique combination of characteristics. If a concept does not specify a TextFormat or a core representation, then the representation of the concept is assumed to be represented by any set of valid characters (corresponding to the xs:string datatype of W3C XML Schema).</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="ConceptBaseType">
				<xs:sequence>
					<xs:element name="CoreRepresentation" type="ConceptRepresentation" minOccurs="0">
						<xs:annotation>
							<xs:documentation></xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element name="ISOConceptReference" type="ISOConceptReferenceType" minOccurs="0">
						<xs:annotation>
							<xs:documentation>Provides a reference to an ISO 11179 concept.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
