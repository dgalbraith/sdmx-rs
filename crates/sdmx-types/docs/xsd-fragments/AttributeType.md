<details>
<summary>XSD contract: <code>AttributeType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="AttributeType">
		<xs:annotation>
			<xs:documentation>AttributeType describes the structure of a data attribute, which is defined as a characteristic of an object or entity. The attribute takes its semantic, and in some cases it representation, from its concept identity. An attribute can be coded by referencing a code list from its coded local representation. It can also specify its text format, which is used as the representation of the attribute if a coded representation is not defined. Neither the coded or uncoded representation are necessary, since the attribute may take these from the referenced concept. An attribute specifies its relationship with other data structure components and is given an assignment status. These two properties dictate where in a data message the attribute will be attached, and whether or not the attribute will be required to be given a value. A set of roles defined in concept scheme can be assigned to the attribute.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="AttributeBaseType">
				<xs:sequence>
					<xs:element name="ConceptRole" type="common:ConceptReferenceType" minOccurs="0" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>ConceptRole references concepts which define roles which this attribute serves.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element name="AttributeRelationship" type="AttributeRelationshipType">
						<xs:annotation>
							<xs:documentation>AttributeRelationship describes how the value of this attribute varies with the values of other components. These relationships will be used to determine the attachment level of the attribute in the various data formats.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element name="MeasureRelationship" type="MeasureRelationshipType" minOccurs="0">
						<xs:annotation>
							<xs:documentation>MeasureRelationship identifies the measures that the attribute applies to. If this is not used, the attribute is assumed to apply to all measures.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
				<xs:attribute name="usage" type="UsageType" use="optional" default="optional">
					<xs:annotation>
						<xs:documentation>The usage attribute indicates whether an attribute value must be available for any corresponding existing observation.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
