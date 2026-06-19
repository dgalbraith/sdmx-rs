<details>
<summary>XSD contract: <code>MeasureType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="MeasureType">
		<xs:annotation>
			<xs:documentation>MeasureType defines the structure of a measure descriptor. In addition to the identifying concept and representation, a usage status and max occurs can be defined.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="MeasureBaseType">
				<xs:sequence>
					<xs:element name="ConceptRole" type="common:ConceptReferenceType" minOccurs="0" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>ConceptRole references concepts which define roles which this measure serves.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
				<xs:attribute name="usage" type="UsageType" use="optional" default="optional">
					<xs:annotation>
						<xs:documentation>The usage attribute indicates whether a measure value must be available for any corresponding existing observation.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
